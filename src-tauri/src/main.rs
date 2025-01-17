#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;
use std::thread;
use tauri::Manager;
use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use futures_util::{StreamExt, SinkExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;

mod convert;

lazy_static! {
    static ref STOP_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref FRAME_BUFFER: Arc<Mutex<Vec<(Vec<u8>, Duration, u32, u32)>>> = Arc::new(Mutex::new(Vec::new()));
    static ref CAPTURING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

#[tauri::command]
fn stop_recording() {
    STOP_FLAG.store(true, Ordering::Release);
    println!("Stopped recording");
}

fn capture_frames(width: u32, height: u32) -> Result<(), String> {
    let monitors = Display::all().unwrap();

    let monitor = if monitors.is_empty() {
        return Err("No monitor displays found".into());
    } else {
        monitors.into_iter().nth(0).unwrap()
    };
    let mut capturer = Capturer::new(monitor).map_err(|e| e.to_string())?;
    let start = Instant::now();

    let mut prev_frame: Option<Vec<u8>> = None;

    while !STOP_FLAG.load(Ordering::Acquire) {
        let now = Instant::now();
        let time = now - start;

        match capturer.frame() {
            Ok(frame) => {
                let frame_data = frame.to_vec();
                let is_different = if let Some(prev) = &prev_frame {
                    is_frame_different(prev, &frame_data)
                } else {
                    true
                };

                if is_different {
                    prev_frame = Some(frame_data.clone());
                    println!("captured frame, {}", time.as_secs() * 1_000 + time.subsec_millis() as u64);
                    FRAME_BUFFER.lock().unwrap().push((frame_data, time, width, height));
                }
            },
            Err(ref e) if e.kind() == WouldBlock => {
                // Just wait
            },
            Err(e) => {
                return Err(e.to_string());
            }
        }

        let seconds_per_frame = Duration::from_nanos(1_000_000_000 / 24);
        let dt = now.elapsed();
        if dt < seconds_per_frame {
            thread::sleep(seconds_per_frame - dt);
        }
    }

    Ok(())
}

fn is_frame_different(prev: &[u8], current: &[u8]) -> bool {
    if prev.len() != current.len() {
        return true;
    }

    const DIFFERENCE_THRESHOLD: usize = 10_000; // Adjust the threshold as needed
    let mut diff_count = 0;

    for (p, c) in prev.iter().zip(current.iter()) {
        if p != c {
            diff_count += 1;
            if diff_count > DIFFERENCE_THRESHOLD {
                return true;
            }
        }
    }
    false
}

async fn emit_frames(mut ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>) {
    while CAPTURING.load(Ordering::Acquire) {
        // Clone the buffer contents and release the lock immediately
        let frames: Vec<(Vec<u8>, Duration, u32, u32)> = {
            let buffer = FRAME_BUFFER.lock().unwrap();
            let frames = buffer.clone();
            frames
        };

        // Emit each frame in the cloned buffer
        for (frame_data, time, width, height) in frames {
            let payload = serde_json::json!({
                "data": frame_data,
                "pts": time.as_secs() * 1_000 + time.subsec_millis() as u64,
                "width": width,
                "height": height
            }).to_string();
            match app_handle.emit_all("frame", payload) {
                Ok(_) => println!("emitting frame, {}", time.as_secs() * 1_000 + time.subsec_millis() as u64),
                Err(e) => eprintln!("Failed to emit frame: {}", e)
            }
        }

        // Sleep to avoid busy-waiting
        tokio::time::sleep(Duration::from_millis(10)).await; // Adjust the sleep duration as needed
    }
}

#[tauri::command]
async fn start_recording(app_handle: tauri::AppHandle) -> Result<(), String> {
    let monitors = Display::all().unwrap();

    let monitor = if monitors.is_empty() {
        return Err("No monitor displays found".into());
    } else {
        monitors.into_iter().nth(0).unwrap()
    };

    let width = monitor.width() as u32;
    let height = monitor.height() as u32;

    STOP_FLAG.store(false, Ordering::Release);
    CAPTURING.store(true, Ordering::Release);
    println!("Started Recording");

    let app_handle_clone = app_handle.clone();

    thread::spawn(move || {
        if let Err(e) = capture_frames(width, height) {
            eprintln!("Capture error: {}", e);
        }
        CAPTURING.store(false, Ordering::Release);
    });

    Ok(())
}

fn main() {
    tauri::async_runtime::spawn(start_server());
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_recording, stop_recording])
        .plugin(tauri_plugin_websocket::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
