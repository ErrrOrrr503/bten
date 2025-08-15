#!/usr/bin/env python3
"""
A tiny utility that sends a 4‑byte command on a serial port.

Usage:
    python3 script.py on     # Sends 0x42 0x00 0x01 0x24
    python3 script.py off    # Sends 0x42 0x00 0x00 0x24
"""

import sys
import serial

# ------------------------------------------------------------------
#  1.  CONFIGURATION ------------------------------------------------
# ------------------------------------------------------------------
PORT = "/dev/ttyUSB0"

BAUDRATE = 115200
TIMEOUT  = 1

CMD_MAGIC_START = 0x42
CMD_MAGIC_END = 0x24
STATUS_MAGIC_START = 0x32
STATUS_MAGIC_END = 0x23

CMD_CODES = {
    "off": 0,
    "on": 1,
    "reboot": 2,
    "status": 3
}

NUM_PORTS = 1

VERBOSE = True

# ------------------------------------------------------------------
#  2.  INPUT PARSING ------------------------------------------------
# ------------------------------------------------------------------
def print_usage_exit():
    print("Usage: {} <on|off|reboot|status> <port: 0..{}>".format(sys.argv[0], NUM_PORTS-1))
    exit(0)

try:
    if len(sys.argv) != 3:
        raise ValueError
    action = sys.argv[1]
    if action not in CMD_CODES.keys():
        raise ValueError
    port = int(sys.argv[2])
    if port < 0 or port >= NUM_PORTS:
        raise ValueError
except ValueError:
    print_usage_exit()


# ------------------------------------------------------------------
#  3.  BUILD THE COMMAND --------------------------------------------
# ------------------------------------------------------------------
command = bytes([CMD_MAGIC_START, port, CMD_CODES[action], CMD_MAGIC_END])

# ------------------------------------------------------------------
#  4.  SEND THE COMMAND ---------------------------------------------
# ------------------------------------------------------------------

def serial_cmd():
    with serial.Serial(PORT, BAUDRATE, timeout=TIMEOUT) as ser:
        ser.write(command)
        ser.flush()
    if VERBOSE:
        print("Sent: {:02X} {:02X} {:02X} {:02X}".format(*command))
    if action == "status":
        bstatus = b''
        with serial.Serial(PORT, BAUDRATE, timeout=TIMEOUT) as ser:
            byte = ser.read()
            while byte != STATUS_MAGIC_START:
                byte = ser.read()
            byte = ser.read()
            while byte != STATUS_MAGIC_END:
                bstatus += byte
                byte = ser.read()
        print(bstatus.decode('utf-8'))


try:
    serial_cmd()
except FileNotFoundError:
    print("Error: Could not open serial port '{}'".format(PORT), file=sys.stderr)
    sys.exit(1)
except serial.SerialException as e:
    print("Serial error: {}".format(e), file=sys.stderr)
    sys.exit(1)