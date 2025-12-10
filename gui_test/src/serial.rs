use lazy_static::lazy_static;
use parking_lot::Mutex;
use serialport::{self, SerialPort, SerialPortInfo};
use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;
/*
* Available Serial Ports:
- /dev/cu.debug-console
- /dev/tty.debug-console
- /dev/cu.usbmodem31111405 (USB VID:046d PID:0ac4)
- /dev/tty.usbmodem31111405 (USB VID:046d PID:0ac4)
- /dev/cu.Bluetooth-Incoming-Port
- /dev/tty.Bluetooth-Incoming-Port
- /dev/cu.usbmodem311402 (USB VID:0483 PID:3753)
- /dev/tty.usbmodem311402 (USB VID:0483 PID:3753)
- /dev/cu.usbmodem311405 (USB VID:0483 PID:3753)
- /dev/tty.usbmodem311405 (USB VID:0483 PID:3753)
- /dev/cu.usbserial-0001 (USB VID:10c4 PID:ea60)
- /dev/tty.usbserial-0001 (USB VID:10c4 PID:ea60)
*/

use crate::{AppState, EguiApp};
use std::time::Duration; // Need to import Read trait for the read_exact method

// const PORT_NAME: &str = "/dev/cu.usbserial-0001"; // Your device's port
const BAUD_RATE: u32 = 115200;
const SERIAL_TIMEOUT_MS: u64 = 1000;

fn open_serial_port(port_name: &str, baud_rate: u32) -> serialport::Result<Box<dyn SerialPort>> {
    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(SERIAL_TIMEOUT_MS))
        .open()?; // The '?' operator returns the error if the port can't be opened

    Ok(port)
}

// Sets the maximum expected response size. Adjust this based on your device.
const MAX_RESPONSE_SIZE: usize = 256;

pub fn list_available_serial_ports() -> Result<Vec<String>, String> {
    // 1. Call the function to get all available serial ports.
    let ports = match serialport::available_ports() {
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to list ports: {}", e)),
    };

    // 2. Process the list to extract the port names and specific details.
    if ports.is_empty() {
        return Ok(vec!["No serial ports found.".to_string()]);
    }

    let port_names: Vec<String> = ports
        .into_iter()
        .map(|p: SerialPortInfo| {
            // Optional: Include device information if it's a USB port
            let details = match p.port_type {
                serialport::SerialPortType::UsbPort(usb_info) => {
                    format!(" (USB VID:{:04x} PID:{:04x})", usb_info.vid, usb_info.pid)
                }
                _ => String::new(),
            };
            format!("{}{}", p.port_name, details)
        })
        .collect();

    Ok(port_names)
}

pub fn send_and_receive_serial_message(
    mut port: Box<dyn SerialPort>,
    message: &str,
) -> serialport::Result<String> {
    // Use format! to include common command terminators (\r\n)
    let message_with_terminator = format!("{}\r\n", message);
    let bytes_to_send = message_with_terminator.as_bytes();

    port.write(bytes_to_send)?;
    println!("Attempting to send: '{message}'");
    let mut reader = BufReader::new(&mut port);
    let mut read_buffer: Vec<u8> = Vec::new();

    // Read until the device sends a newline character (b'\n')
    let bytes_read = match reader.read_until(b'\n', &mut read_buffer) {
        Ok(0) => {
            return Err(serialport::Error::new(
                serialport::ErrorKind::NoDevice,
                "Connection closed or device sent no data.",
            ));
        }
        Ok(t) => t,
        Err(e) => {
            if e.kind() == io::ErrorKind::TimedOut {
                // If it times out, handle it gracefully by returning a specific error.
                // Since serialport::ErrorKind lacks a Timeout variant, we can use a descriptive Io variant.
                return Err(serialport::Error::new(
                    serialport::ErrorKind::Io(io::ErrorKind::TimedOut),
                    "Read operation timed out.",
                ));
            }

            // Handle all other fatal I/O errors
            return Err(serialport::Error::new(
                serialport::ErrorKind::Io(e.kind()),
                format!("Error during read operation: {}", e),
            ));
        }
    };

    println!("Received {} bytes in response.", bytes_read);

    // Convert the received bytes (up to bytes_read) into a UTF-8 String.
    let received_bytes = &read_buffer[..bytes_read];

    // Attempt to convert the received byte slice into a readable string
    match String::from_utf8(received_bytes.to_vec()) {
        Ok(s) => Ok(s.trim().to_string()), // Trim whitespace/newlines and return
        Err(e) => {
            // This 'Err' handles only UTF-8 errors, not I/O errors.
            return Err(serialport::Error::new(
                serialport::ErrorKind::Io(io::ErrorKind::InvalidData),
                format!("Response data was not valid UTF-8: {}", e),
            ));
        }
    }
}

// 1. Define the static variable, protected by a Mutex.
lazy_static! {
    pub static ref T10_DETECTED: Mutex<bool> = Mutex::new(false);
}

/// This function sends "m ver" command looking for a valid response.
/// If it finds it, it updates the AppState value and the port_name
/// so the GUI reacts accordingly.
pub fn find_t10(
    state_arc_clone: Arc<Mutex<AppState>>,
    port_name_arc_clone: Arc<Mutex<String>>,
) -> bool {
    // Listing all available ports ...
    let mut serial_ports: String = String::new();
    match list_available_serial_ports() {
        Ok(port_list) => {
            // println!("port_list = {:?}", port_list);
            serial_ports.push_str("Available Serial Ports:\n");
            for port in port_list {
                if port.contains("/dev/cu.usb") {
                    serial_ports.push_str(&format!("- {}\n", port));
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    };

    if serial_ports.len() == 0 {
        println!("No usb serial devices connected");
        return false;
    } else {
        // Check every device:
        for serial_port in serial_ports.split("\n") {
            let port_name = serial_port.trim().split_whitespace().nth(1).unwrap_or("");
            println!("Attempting to communicate with {port_name}");
            match open_serial_port(port_name, BAUD_RATE) {
                Ok(serial_port) => {
                    let detected = {
                        let serial_response =
                            match send_and_receive_serial_message(serial_port, "m ver") {
                                Ok(response) => response,
                                Err(err) => format!("Error: {}", err),
                            };

                        println!("Serial Response: {}", serial_response);

                        if !serial_response.contains("Error") {
                            println!("T10 detected!");
                            // Updating app state:
                            let mut guard = state_arc_clone.lock();
                            *guard = AppState::SerialComState;

                            // Updating port_name:
                            let mut port_guard = port_name_arc_clone.lock();
                            *port_guard = port_name.to_owned();
                            true
                        } else {
                            false
                        }
                    };
                    if detected {
                        return true;
                    }
                }
                Err(err) => println!("Error opening serial port: {}", err),
            }
        }
        println!("T10 not detected ...");
        return false;
    }
}

pub fn send_serial_command(
    port_name: Arc<Mutex<String>>,
    message_to_send: &str,
    serial_response: Arc<Mutex<String>>,
) {
    let port_name_clone = port_name.lock().clone();
    // let port_name_clone = *port_guard;
    println!("Sending '{}' to {}.", message_to_send, port_name_clone);
    match open_serial_port(&port_name_clone, BAUD_RATE) {
        Ok(serial_port) => {
            // println!("Serial Port opened");
            let result = match send_and_receive_serial_message(serial_port, message_to_send) {
                Ok(response) => response,
                Err(err) => format!("Error: {}", err),
            };
            println!("Serial Response: {}", result);
            let mut guard = serial_response.lock();
            *guard = result;
        }
        Err(err) => println!("Error opening serial port: {}", err),
    }
}
