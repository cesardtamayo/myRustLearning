use anyhow::Result;
use eframe::egui;
use egui::{Button, Color32, RichText, TextEdit, Ui};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use serialport::{self, SerialPort, SerialPortInfo};
use std::thread;

use std::io::{self, BufRead, BufReader, Read, Write}; // Need BufReader and BufRead for read_until

use std::time::Duration; // Need to import Read trait for the read_exact method

const PORT_NAME: &str = "/dev/cu.usbserial-0001"; // Your device's port
const BAUD_RATE: u32 = 115200;
const SERIAL_TIMEOUT_MS: u64 = 3000;

fn open_serial_port(port_name: &str, baud_rate: u32) -> serialport::Result<Box<dyn SerialPort>> {
    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(SERIAL_TIMEOUT_MS))
        .open()?; // The '?' operator returns the error if the port can't be opened

    Ok(port)
}

// Sets the maximum expected response size. Adjust this based on your device.
const MAX_RESPONSE_SIZE: usize = 256;

pub fn send_and_receive_serial_message(
    mut port: Box<dyn SerialPort>,
    message: &str,
) -> serialport::Result<String> {
    // --- 1. PREPARE AND SEND THE MESSAGE ---

    // Use format! to include common command terminators (\r\n)
    let message_with_terminator = format!("{}\r\n", message);
    let bytes_to_send = message_with_terminator.as_bytes();

    println!("Attempting to send: '{}'", message);

    // Write the byte slice to the serial port
    port.write(bytes_to_send)?;

    // --- 2. WAIT FOR AND READ THE RESPONSE ---

    // Initialize a buffer to hold the incoming bytes.
    // let mut read_buffer: Vec<u8> = vec![0; MAX_RESPONSE_SIZE];

    let mut reader = BufReader::new(&mut port);
    let mut read_buffer: Vec<u8> = Vec::new();

    // 3b. Read until the device sends a newline character (b'\n')
    let bytes_read = match reader.read_until(b'\n', &mut read_buffer) {
        Ok(0) => {
            return Err(serialport::Error::new(
                serialport::ErrorKind::NoDevice,
                "Connection closed or device sent no data.",
            ));
        }
        Ok(t) => t,
        Err(e) => {
            // Map standard I/O errors (other than timeout) to serialport::Error
            return Err(serialport::Error::new(
                serialport::ErrorKind::Io(e.kind()),
                format!("Error during read operation: {}", e),
            ));
        }
    };
    // let bytes_read = match port.read(&mut read_buffer) {
    //     Ok(t) => t,
    //     Err(e) => {
    //         return Err(serialport::Error::new(
    //             // 1. Use the 'Io' variant from the serialport crate
    //             serialport::ErrorKind::Io(
    //                 // 2. Pass the correct standard library error kind
    //                 io::ErrorKind::InvalidData,
    //             ),
    //             format!("Error reading response: {}", e),
    //         ));
    //     }
    // };

    println!("Received {} bytes in response.", bytes_read);

    // --- 3. CONVERT BYTES TO STRING ---

    // Convert the received bytes (up to bytes_read) into a UTF-8 String.
    let received_bytes = &read_buffer[..bytes_read];

    // Attempt to convert the received byte slice into a readable string
    match String::from_utf8(received_bytes.to_vec()) {
        Ok(s) => Ok(s.trim().to_string()), // Trim whitespace/newlines and return
        Err(e) => {
            // Return an error if the received data is not valid UTF-8
            Err(serialport::Error::new(
                // 1. Use the 'Io' variant from the serialport crate
                serialport::ErrorKind::Io(
                    // 2. Pass the correct standard library error kind
                    io::ErrorKind::InvalidData,
                ),
                format!("Response data was not valid UTF-8: {}", e),
            ))
        }
    }
}

// 1. Define the static variable, protected by a Mutex.
lazy_static! {
    pub static ref DEVICE_SCAN_OUTPUT: Mutex<String> =
        Mutex::new(String::from("Device scan has not run yet..."));
}

// Assume this function runs your device scanning logic
pub fn scan_devices() {
    // 1. Placeholder for your actual slow scan logic
    let mut results: String = String::new();
    match list_available_serial_ports() {
        Ok(port_list) => {
            results.push_str("Available Serial Ports:\n");
            for port in port_list {
                results.push_str(&format!("- {}\n", port));
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    // let results: String = list_usb_devices().expect("Failed to scan devices");

    // 2. Lock and update the shared static variable
    let mut data = crate::DEVICE_SCAN_OUTPUT.lock();
    *data = results;

    // The lock is released automatically here
    println!("Device scan completed");
}

/*
 *
 * Find connected taser 10 by sending "m_ver" to all connected devices.
 * The T10 will return "mayor.minor.patch" string.
 *
*/

const APP_NAME: &str = "My First Rust App";
const WINDOW_WIDTH: f32 = 400.0;
const WINDOW_HEIGHT: f32 = 400.0;
const BUTTON_SIZE: f32 = 18.0;
const HEADING_FONT_SIZE: f32 = 25.0;
const LABEL_FONT_SIZE: f32 = 15.0;
const BIGGER_SPACING_SIZE: f32 = 10.0;
const SMALLER_SPACING_SIZE: f32 = 5.0;

#[derive(Default)]
struct EguiApp {
    state: AppState,
}

#[derive(Default, Debug, PartialEq, Eq)]
enum AppState {
    #[default]
    InitialState,
}

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

fn main() {
    // let options = eframe::NativeOptions {
    //     viewport: egui::ViewportBuilder::default()
    //         .with_inner_size(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT)),
    //     ..Default::default()
    // };
    // eframe::run_native(
    //     APP_NAME,
    //     options,
    //     Box::new(|cc| Ok(Box::new(EguiApp::new(cc)))),
    // )
    // .expect("Failed to app");
    let message_to_send = "m ver";
    match open_serial_port(PORT_NAME, BAUD_RATE) {
        Ok(serial_port) => {
            let serial_response =
                match send_and_receive_serial_message(serial_port, message_to_send) {
                    Ok(response) => response,
                    Err(err) => format!("Error: {}", err),
                };
            println!("Serial Response: {}", serial_response);
        }
        Err(err) => println!("Error opening serial port: {}", err),
    }
}

impl EguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
    fn change_state(&mut self, new_state: AppState) {
        self.state = new_state;
    }
}
impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let result = match self.state {
            AppState::InitialState => show_home_screen(self, ctx),
            // AppState::ScanningDevices => show_scanning_screen(self, ctx),
        };
        if result.is_err() {
            println!("Error occured!")
        }
    }
}

fn show_home_screen(app: &mut EguiApp, ctx: &egui::Context) -> Result<()> {
    egui::CentralPanel::default().show(ctx, |ui| -> Result<()> {
        get_home_screen(app, ui)?;
        Ok(())
    });
    Ok(())
}

fn get_home_screen(app: &mut EguiApp, ui: &mut Ui) -> Result<()> {
    ui.heading(RichText::new(APP_NAME).size(HEADING_FONT_SIZE).strong());
    ui.add_space(BIGGER_SPACING_SIZE);
    ui.label(
        RichText::new("Scanning devices ...".to_string())
            .size(LABEL_FONT_SIZE)
            .color(Color32::YELLOW),
    );
    ui.add_space(SMALLER_SPACING_SIZE);

    if ui
        .add(Button::new(RichText::new("Scan devices").size(BUTTON_SIZE)))
        .clicked()
    {
        thread::spawn(|| {
            scan_devices();
        });
    }

    // 1. Lock the Mutex to read the shared String
    let current_output = DEVICE_SCAN_OUTPUT.lock();

    // 2. Get a reference to the String data
    let display_text: &str = current_output.as_str();

    // 3. Display the text in a multi-line, read-only text box
    let mut binding = display_text.to_owned();
    let text_box = TextEdit::multiline(&mut binding) // Use .to_owned() for TextEdit
        .desired_rows(10)
        .frame(true)
        .interactive(false); // Make it read-only

    ui.add_sized([ui.available_width(), 200.0], text_box);

    Ok(())
}

// use rusb::{Device, DeviceList, UsbContext};

// pub fn list_usb_devices() -> rusb::Result<String> {
//     // 1. Initialize a mutable String to collect all output.
//     let mut output = String::new();

//     let context = rusb::Context::new()?;
//     let devices: DeviceList<rusb::Context> = context.devices()?;

//     // 2. Use format! (or writeln! with a separate String buffer) instead of println!
//     let header = format!("🔎 Found {} USB Devices:\n", devices.len());
//     output.push_str(&header);

//     // Iterate over the devices
//     for device in devices.iter() {
//         let device_desc = device.device_descriptor()?;

//         // 3. Use format! to create the line string
//         let line = format!(
//             "  - Bus {:03} Address {:03} | ID {:04x}:{:04x}\n",
//             device.bus_number(),
//             device.address(),
//             device_desc.vendor_id(),
//             device_desc.product_id()
//         );

//         // 4. Append the formatted line to the output String
//         output.push_str(&line);
//     }

//     // 5. Return the collected String wrapped in Ok() on success.
//     // The error type is automatically inferred as rusb::Error based on the '?' operator.
//     Ok(output)
// }
