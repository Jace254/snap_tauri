#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::{Instant, Duration};
use std::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;
use std::sync::Arc;
use std::thread;
use tauri::Manager;
use xcap::Monitor;

mod convert;

lazy_static! {
    static ref STOP_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

#[tauri::command]
fn stop_recording() {
    STOP_FLAG.store(true, Ordering::Release);
    println!("Stopped recording")
}


#[tauri::command]
async fn start_recording(app_handle: tauri::AppHandle) -> Result<(), String> {
    let monitors = Monitor::all().unwrap();

    let monitor = if monitors.is_empty() {
        return Err("No monitor displays found".into());
    } else {
        monitors.into_iter().nth(0).unwrap()
    };

    let width = monitor.width();
    let height = monitor.height();

    let start = Instant::now();
    STOP_FLAG.store(false, Ordering::Release);
    let app_handle_clone = app_handle.clone();
    println!("Started Recording");

    let seconds_per_frame = Duration::from_nanos(1_000_000_000 / 240);
    // let mut yuv = Vec::new();


    while !STOP_FLAG.load(Ordering::Acquire) {
        let now = Instant::now();
        let time = now - start;

        match monitor.capture_image() {
            Ok(frame) => {
                // convert::argb_to_i420(&frame, &mut yuv);               
                let payload = serde_json::json!({
                    "data": frame.into_raw(),
                    "pts": time.as_secs() * 1_000 + time.subsec_millis() as u64,
                    "width": width,
                    "height": height
                });
                app_handle_clone.emit_all("frame", payload.to_string()).unwrap();
            },
            Err(e) => {
                println!("{}", e);
                break;
            }
        }

        let dt = now.elapsed();
        if dt < seconds_per_frame {
            thread::sleep(seconds_per_frame - dt);
        }
    }


    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_recording, stop_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
