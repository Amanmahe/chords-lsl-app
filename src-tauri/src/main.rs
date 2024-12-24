use serialport;
use std::io::{self, Read, Write};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{self, AppHandle, Emitter}; // Import Emitter along with AppHandle


use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
lazy_static! {
    static ref BAUDRATE: Arc<Mutex<u32>> = Arc::new(Mutex::new(230400)); // Default baud rate
    static ref PACKET_SIZE: Arc<Mutex<usize>> = Arc::new(Mutex::new(16)); // Default baud rate
    static ref CHANNELS: Arc<Mutex<usize>> = Arc::new(Mutex::new(6)); // Default baud rate
    static ref SAMPLE_RATE: Arc<Mutex<f64>> = Arc::new(Mutex::new(500.0)); // Default baud rate
}


#[derive(Clone, serde::Serialize)]
struct Payload {
    message: Vec<i16>,
}
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

#[tauri::command]
fn auto_detect_arduino() -> Result<String, String> {
    loop {
        let ports = serialport::available_ports().expect("No ports found!");
        let baud_rate = *BAUDRATE.lock().unwrap(); // Lock and retrieve the current baud rate

        for port_info in ports {
            let port_name = port_info.port_name;
            println!("Attempting to connect to port: {}", port_name);
            if let serialport::SerialPortType::UsbPort(info) = port_info.port_type {
                println!(
                    "Port: {} has VID: {:04X} and PID: {:04X}",
                    port_name, info.vid, info.pid
                );
                print_type_of(&info.pid);
                // Check if the VID and PID match your Arduino device
                if info.vid == 6790 {//MAKER UNO
                    *BAUDRATE.lock().unwrap() = 115200; // Change the baud rate dynamically
                    *SAMPLE_RATE.lock().unwrap() = 250.0; 
                } 
                if info.pid == 67 {//Arduino UNO R3               {
                    *SAMPLE_RATE.lock().unwrap() = 250.0; 
                } 
                if info.vid == 11914 {//Resberry pi pico
                    *PACKET_SIZE.lock().unwrap() = 10; // Change the baud rate dynamically
                    *CHANNELS.lock().unwrap() = 3; // Change the baud rate dynamically
                } 
            }
            println!("baudrate: {} ", *BAUDRATE.lock().unwrap());

            match serialport::new(&port_name, *BAUDRATE.lock().unwrap())
                .timeout(Duration::from_secs(3))
                .open()
            {
                Ok(mut port) => {
                    thread::sleep(Duration::from_secs(3)); // Allow Arduino to reset
                    let command = b"WHORU\n";

                    if let Err(e) = port.write_all(command) {
                        println!("Failed to write to port: {}. Error: {:?}", port_name, e);
                        continue;
                    }
                    port.flush().expect("Failed to flush port");
                    println!("Sending command...");

                    let mut buffer: Vec<u8> = vec![0; 1024];
                    let mut response = String::new();
                    let start_time = Instant::now();
                    let timeout = Duration::from_secs(10);

                    while start_time.elapsed() < timeout {
                        match port.read(&mut buffer) {
                            Ok(size) => {
                                if size > 0 {
                                    response.push_str(&String::from_utf8_lossy(&buffer[..size]));
                                    println!("Partial response: {}", response);

                                    if response.contains("UNO-R4")
                                        || response.contains("UNO-R3")
                                        || response.contains("GIGA-R1")
                                        || response.contains("RPI-PICO-RP2040")
                                        || response.contains("UNO-CLONE")
                                    {
                                        println!("Valid device found on port: {}", port_name);
                                        drop(port);
                                        return Ok(port_name); // Return the found port name directly
                                    }
                                }
                            }
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
                            Err(e) => {
                                println!("Failed to read from port: {}. Error: {:?}", port_name, e);
                                break;
                            }
                        }
                    }

                    println!("Final response from port {}: {}", port_name, response);

                    drop(port);
                }
                Err(e) => {
                    println!("Failed to open port: {}. Error: {:?}", port_name, e);
                }
            }
        }

        println!("No valid device found, retrying in 5 seconds...");
        thread::sleep(Duration::from_secs(5)); // Wait before trying again
    }
}

#[tauri::command]
fn monitor_device_connection(port_name: String, app_handle: AppHandle) {
    use lsl::{Pushable, StreamInfo, StreamOutlet};
    use std::{
        io,
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    // Constants for packet handling
    const START_BYTE_1: u8 = 0xC7;
    const START_BYTE_2: u8 = 0x7C;
    const END_BYTE: u8 = 0x01;

    // Create an LSL stream
    let info = lsl::StreamInfo::new(
        "UDL",
        "Biopotential_Signals",
        (*CHANNELS.lock().unwrap()).try_into().unwrap(),
        *SAMPLE_RATE.lock().unwrap(),
        lsl::ChannelFormat::Int16,
        "unique_id_12345",
    )
    .unwrap();

    let outlet = Arc::new(Mutex::new(StreamOutlet::new(&info, 0, 360).unwrap()));

    println!("{}gg", *SAMPLE_RATE.lock().unwrap());

    loop {
        match serialport::new(&port_name, *BAUDRATE.lock().unwrap())
            .timeout(Duration::from_secs(3))
            .open()
        {
            Ok(mut port) => {

                println!("Connected to device on port: {}", port_name);
                let start_command = b"START\r\n";
 
                for i in 1..=3 {
                    if let Err(e) = port.write_all(start_command) {
                        println!("Failed to send START command (attempt {}): {:?}", i, e);
                    } else {
                        println!("START command sent (attempt {}).", i);
                    }
                    thread::sleep(Duration::from_millis(400)); // Optional: Delay between commands
                }

                println!("Finished sending commands.");

                let mut buffer: Vec<u8> = vec![0; 1024];
                let mut accumulated_buffer: Vec<u8> = Vec::new();

                loop {
                    match port.read(&mut buffer) {
                        Ok(size) => {
                            accumulated_buffer.extend_from_slice(&buffer[..size]);

                            // Process packets if we have enough bytes
                            while accumulated_buffer.len() >= *PACKET_SIZE.lock().unwrap() {
                                if accumulated_buffer[0] == START_BYTE_1
                                    && accumulated_buffer[1] == START_BYTE_2
                                {
                                    if accumulated_buffer[*PACKET_SIZE.lock().unwrap()  - 1] == END_BYTE {
                                        // Extract the packet
                                        let packet = accumulated_buffer
                                            .drain(..*PACKET_SIZE.lock().unwrap())
                                            .collect::<Vec<u8>>();

                                        let data: Vec<i16> = (0..*CHANNELS.lock().unwrap())
                                            .map(|i| {
                                                let idx = 3 + (i * 2);
                                                let high = packet[idx] as i16;
                                                let low = packet[idx + 1] as i16;
                                                (high << 8) | low
                                            })
                                            .collect();

                                        let mut data = data;
                                        println!("{:?}", data);

                                        // Emit the data to the frontend
                                        app_handle
                                            .emit(
                                                "updateSerial",
                                                Payload {
                                                    message: data.clone(),
                                                },
                                            )
                                            .unwrap();

                                        // Send the data to LSL
                                        if let Ok(outlet) = outlet.lock() {
                                            outlet.push_sample(&data).unwrap_or_else(|e| {
                                                println!("Failed to push data to LSL: {:?}", e);
                                            });
                                        }
                                    } else {
                                        accumulated_buffer.drain(..1); // Invalid end byte, skip the packet
                                    }
                                } else {
                                    accumulated_buffer.drain(..1); // Invalid start bytes, skip
                                }
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                            println!("Read timed out, retrying...");
                            continue;
                        }
                        Err(e) => {
                            println!("Error receiving data: {:?}", e);
                            break; // Exit the loop on error
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to connect to device on {}: {}", port_name, e);
                break; // Exit the loop on error
            }
        }

        println!("Device disconnected, checking for new devices...");
        thread::sleep(Duration::from_secs(5)); // Wait before trying again
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            auto_detect_arduino,
            monitor_device_connection
        ])
        .setup(|_app| {
            println!("Starting auto-detection of Arduino...");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
