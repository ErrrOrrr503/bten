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
# Change this to the correct device you want to talk to.
# Examples on Linux/macOS:  "/dev/ttyUSB0", "/dev/ttyACM0"
# Examples on Windows:     "COM3", "COM4"
PORT = "/dev/ttyUSB0"          # <- Edit this line

# Typical serial settings – adjust if your device requires something else
BAUDRATE = 115200
TIMEOUT  = 1          # seconds

# ------------------------------------------------------------------
#  2.  INPUT PARSING ------------------------------------------------
# ------------------------------------------------------------------
if len(sys.argv) != 2 or sys.argv[1].lower() not in {"on", "off"}:
    print("Usage: {} <on|off>".format(sys.argv[0]), file=sys.stderr)
    sys.exit(1)

# ------------------------------------------------------------------
#  3.  BUILD THE COMMAND --------------------------------------------
# ------------------------------------------------------------------
# Start marker      : 0x42
# Status byte (on=01, off=00)
# Stop marker       : 0x24
CMD_HEADER = 0x42
CMD_FOOTER = 0x24

status_byte = 0x01 if sys.argv[1].lower() == "on" else 0x00
command = bytes([CMD_HEADER, 0x00, status_byte, CMD_FOOTER])

# ------------------------------------------------------------------
#  4.  SEND THE COMMAND ---------------------------------------------
# ------------------------------------------------------------------
try:
    with serial.Serial(PORT, BAUDRATE, timeout=TIMEOUT) as ser:
        ser.write(command)
        ser.flush()          # Ensure it is transmitted before we close
    print("Sent: {:02X} {:02X} {:02X} {:02X}".format(*command))
except FileNotFoundError:
    print("Error: Could not open serial port '{}'".format(PORT), file=sys.stderr)
    sys.exit(1)
except serial.SerialException as e:
    print("Serial error: {}".format(e), file=sys.stderr)
    sys.exit(1)