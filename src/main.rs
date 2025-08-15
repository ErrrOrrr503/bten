#![no_std]
#![no_main]

const CMD_MAGIC_START: u8 = 0x42;
const CMD_MAGIC_END: u8 = 0x24;
const STATUS_MAGIC_START: u8 = 0x32;
const STATUS_MAGIC_END: u8 = 0x23;

const CMD_CODE_OFF: u8 = 0;
const CMD_CODE_ON: u8 = 1;
const CMD_CODE_REBOOT: u8 = 2;
const CMD_CODE_STATUS: u8 = 3;

use arduino_hal::prelude::*;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use ufmt::uWrite;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Disable all interrupts
    avr_device::interrupt::disable();

    // Configure watchdog for immediate reset
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let watchdog = dp.WDT;
    let pins = arduino_hal::pins!(dp);
    let mut led = pins.d13.into_output();

    for _ in 0..10 {
        led.set_high();
        arduino_hal::delay_ms(100);
        led.set_low();
        arduino_hal::delay_ms(100);
    }

    unsafe {
        // Start configuration sequence
        watchdog.wdtcsr.write(|w| w.bits(0x18));
        // Set watchdog for 16ms timeout and reset mode
        watchdog.wdtcsr.write(|w| w.bits(0x08));
    }

    // Wait for reset to occur
    loop {
        led.set_high();
        arduino_hal::delay_ms(100);
        led.set_low();
        arduino_hal::delay_ms(100);
    }
}

struct Cmdbuf {
    buf: [u8; 128],
    len: u8,
    pos: u8
}

impl Cmdbuf {
    pub fn new() -> Self {
        Cmdbuf {
            buf: [0;128],
            len: 128,
            pos: 0,
        }
    }
    pub fn append(&mut self, b: u8) -> Result<(), ()> {
        if self.pos >= self.len { return Err(()) }
        self.buf[self.pos as usize] = b;
        self.pos += 1;
        return Ok(());
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    pub fn get(&self) -> &[u8] {
        &self.buf[0..(self.pos as usize)]
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Action {
    On,
    Off,
    Reboot,
    Status,
}

struct Cmd {
    action: Action,
    port: u8,
}

impl Cmd {
    pub fn from_cmdbuf(cmdbuf: &Cmdbuf) -> Result<Self,()> {
        let buf = cmdbuf.get();
        if buf[0] == CMD_MAGIC_START && buf[buf.len() - 1] == CMD_MAGIC_END && buf.len() == 4 {
            return Ok(Self {
                port: buf[1],
                action: match buf[2] {
                    CMD_CODE_OFF => Action::Off,
                    CMD_CODE_ON => Action::On,
                    CMD_CODE_REBOOT => Action::Reboot,
                    CMD_CODE_STATUS => Action::Status,
                    _ => return Err(()),
                },
            })
        } else {
            return Err(());
        }
    }
    pub fn repr(&self) -> &str {
        match self.action {
            Action::Off => "Off",
            Action::On => "On",
            Action::Reboot => "Reboot",
            Action::Status => "Status",
        }
    }
}

struct PortManager<PIN0: OutputPin, PIN1: OutputPin, SERIAL>
where
    SERIAL: _embedded_hal_serial_Read<u8> + uWrite,
{
    pin0: PIN0,
    pin1: PIN1,
    serial: SERIAL
}

impl<PIN0: StatefulOutputPin, PIN1: StatefulOutputPin, SERIAL> PortManager<PIN0, PIN1, SERIAL>
where
    SERIAL: _embedded_hal_serial_Read<u8> + uWrite,
{
    pub fn new(pin0: PIN0, pin1: PIN1, serial: SERIAL) -> Self {
        Self { pin0, pin1, serial }
    }

    /* following is a crap due to each pin is own type. */
    pub fn process_cmd(&mut self, cmd: &Cmd) -> Result<(), ()> {
        match cmd.action {
            Action::Off => {
                match cmd.port {
                    0 => { self.pin0.set_high().unwrap(); Ok(()) },
                    1 => { self.pin1.set_high().unwrap(); Ok(()) },
                    _ => Err(()),
                }
            },
            Action::On => {
                match cmd.port {
                    0 => { self.pin0.set_low().unwrap(); Ok(()) },
                    1 => { self.pin1.set_low().unwrap(); Ok(()) },
                    _ => Err(()),
                }
            },
            Action::Status => {
                match cmd.port {
                    0 => { let status = if self.pin0.is_set_low().unwrap() { "on" } else { "off" }; self._serial_write_status_str(status); Ok(()) },
                    1 => { let status = if self.pin1.is_set_low().unwrap() { "on" } else { "off" }; self._serial_write_status_str(status); Ok(()) },
                    _ => Err(()),
                }
            },
            Action::Reboot => {
                match cmd.port {
                    0 => { self.pin0.set_high().unwrap(); arduino_hal::delay_ms(5000); self.pin0.set_low().unwrap(); Ok(()) },
                    1 => { self.pin1.set_high().unwrap(); arduino_hal::delay_ms(5000); self.pin1.set_low().unwrap(); Ok(()) },
                    _ => Err(()),
                }
            }
        }
    }

    pub fn serial_read(&mut self) -> u8 {
        match nb::block!(self.serial.read()) {
            Ok(val) => val,
            Err(_) => panic!(),
        }
    }

    pub fn serial_writeln(&mut self, string: &str) {
        match ufmt::uwriteln!(self.serial, "{}", string) {
            Ok(_) => (),
            Err(_) => panic!(),
        }
    }

    pub fn serial_write(&mut self, string: &str) {
        match ufmt::uwrite!(self.serial, "{}", string) {
            Ok(_) => (),
            Err(_) => panic!(),
        }
    }

    pub fn serial_write_byte(&mut self, byte: u8) {
        match self.serial.write_char(byte as char) {
            Ok(_) => (),
            Err(_) => panic!(),
        }
    }

    fn _serial_write_status_str(&mut self, status_str: &str) {
        self.serial_write_byte(STATUS_MAGIC_START);
        self.serial_write(status_str);
        self.serial_write_byte(STATUS_MAGIC_END);
        self.serial_writeln("");
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);

    let switch0 = pins.d12.into_output();
    let switch1 = pins.d11.into_output();

    ufmt::uwriteln!(&mut serial, "aten category /b starting...\r").unwrap_infallible();

    let mut cmdbuf = Cmdbuf::new();

    let mut port_manager = PortManager::new(switch0, switch1, serial);

    loop {
        // Read a byte from the serial connection
        let b = port_manager.serial_read();
        cmdbuf.append(b).unwrap();
        let cmd = Cmd::from_cmdbuf(&cmdbuf);
        if let Ok(cmd) = cmd {
            port_manager.serial_write("Cmd received: ");
            port_manager.serial_write(cmd.repr());
            port_manager.serial_writeln("\r");
            arduino_hal::delay_ms(5000);
            let res = port_manager.process_cmd(&cmd);
            if res == Err(()) {
                port_manager.serial_writeln("    failed!\r");
            } else {
                port_manager.serial_writeln("    succeeded!\r");
            }
            cmdbuf.reset();
        }
    }
}
