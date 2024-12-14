use tauri::Manager;
use lsl::StreamOutlet;

#[tauri::command]
fn start_lsl_stream(app: tauri::AppHandle) -> Result<(), String> {
    let stream_info = lsl::StreamInfo::new("ExampleStream", "EEG", 8, 100.0, lsl::ChannelFormat::Float32, "unique1234")
        .map_err(|e| e.to_string())?;
    let mut outlet = StreamOutlet::new(&stream_info).map_err(|e| e.to_string())?;

    std::thread::spawn(move || {
        let data = vec![0.0; 8]; // 8 channels
        loop {
            // Push sample data to the LSL outlet
            if let Err(err) = outlet.push_sample(&data) {
                eprintln!("Error streaming data: {}", err);
                break;
            }

            // Emit event to the frontend
            app.emit_all("stream-data", format!("{:?}", data))
                .unwrap_or_else(|err| eprintln!("Failed to send event: {}", err));

            std::thread::sleep(std::time::Duration::from_millis(10)); // Adjust for data rate
        }
    });

    Ok(())
}

#[tauri::command]
fn stop_lsl_stream() -> Result<(), String> {
    // Add logic to stop the streaming thread if needed
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_lsl_stream, stop_lsl_stream])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
