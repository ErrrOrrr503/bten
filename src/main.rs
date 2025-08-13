#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;

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
}

const CMD_MAGIC_START: u8 = 0x42;
const CMD_MAGIC_END: u8 = 0x24;

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
                    0 => Action::Off,
                    1 => Action::On,
                    _ => return Err(()),
                },
            })
        } else {
            return Err(());
        }
    }
    pub fn process(&self) -> Result<Action,()> {
        match self.port {
            0 => {},
            _ => return Err(()),
        }; 
        return Ok(self.action);
    }
    pub fn repr(&self) -> &str {
        match self.action {
            Action::Off => "Off",
            Action::On => "On",
        }
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);
    let mut switch = pins.d12.into_output();

    ufmt::uwriteln!(&mut serial, "aten category /b starting...\r").unwrap_infallible();

    let mut cmdbuf = Cmdbuf::new();

    loop {
        // Read a byte from the serial connection
        let b = nb::block!(serial.read()).unwrap_infallible();
        cmdbuf.append(b).unwrap();
        let cmd = Cmd::from_cmdbuf(&cmdbuf);
        if let Ok(cmd) = cmd {
            ufmt::uwriteln!(&mut serial, "Cmd received: {}\r", cmd.repr()).unwrap_infallible();
            arduino_hal::delay_ms(5000);
            let res = cmd.process();
            if res == Err(()) {
                ufmt::uwriteln!(&mut serial, "    failed!\r").unwrap_infallible();
            } else {
                ufmt::uwriteln!(&mut serial, "    succeeded!\r").unwrap_infallible();
                match res {
                    Ok(Action::On) => switch.set_low(),
                    Ok(Action::Off) => switch.set_high(),
                    _ => panic!(),
                }
            }
            cmdbuf.reset();
        }
        // Answer
        // ufmt::uwriteln!(&mut serial, "Got {}!\r", b).unwrap_infallible();
    }
}
