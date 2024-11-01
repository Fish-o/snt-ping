use crate::utils::{create_empty_map_on_heap, Color, HeapMap, Pixel};
use crate::Task;
use chrono::{Local, Timelike};
use image::EncodableLayout;
use image::{GenericImageView, ImageReader};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, mem, thread};


// Dont change these, they should be fine like this.
// You want to make sure the clients remain synchronized
const RESYNC_EVERY: Duration = Duration::from_secs(5 * 60);
const RAND_OFFSET: Duration = Duration::from_secs(15);
const RESYNC_SILENCE: Duration = Duration::from_secs(3);

const MAP_FILE_NAME: &str = "download.png";
pub fn download_new_file(data_pixels: &Arc<Mutex<Vec<Pixel>>>) {
    let lock = data_pixels.lock().expect("Could not lock data_pixels");
    println!("[SYNC] Aquired network lock, waiting for a bit for buffer to empty");
    thread::sleep(RESYNC_SILENCE);
    println!("[SYNC] Downloading map file...");
    let resp = reqwest::blocking::get("https://github.conner.zip/map.png").expect("request failed");
    let body = resp.bytes().expect("body invalid");
    let mut out = File::create(MAP_FILE_NAME).expect("failed to create file");
    io::copy(&mut body.as_bytes(), &mut out).expect("failed to copy content");
    println!(
        "[SYNC] New map file downloaded! (releasing network lock {})",
        lock.len()
    );
}


pub fn synchronize(data_pixels_arc: &Arc<Mutex<Vec<Pixel>>>, map_arc: &Arc<Mutex<HeapMap>>) {
    let img = ImageReader::open(MAP_FILE_NAME)
        .expect("Opening the map image failed :(")
        .decode()
        .expect("Decoding the image failed.");
    assert!(img.width() == 1920, "Image width must be 1920 pixels");
    assert!(img.height() == 1080, "Image height must be 1080 pixels");
    let mut new_data_pixels = vec![];
    let mut new_map: HeapMap = create_empty_map_on_heap();
    let p = img.pixels();
    for (x, y, rgba) in p {
        if rgba.0[3] < 128 {
            new_map[x as usize][y as usize] = None;
            continue;
        }
        new_map[x as usize][y as usize] = Some(Color(rgba.0[0], rgba.0[1], rgba.0[2]));
        new_data_pixels.push(Pixel(x as u16, y as u16));
    }
    new_data_pixels.shuffle(&mut thread_rng());
    println!("[SYNC] Parsed file");

    let mut data_pixels = data_pixels_arc.lock().expect("Could not lock data_pixels");
    println!("[SYNC] Aquired data_pixels lock");
    let mut map = map_arc.lock().expect("Could not lock map");
    println!("[SYNC] Aquired map lock");
    mem::swap(&mut new_map, &mut map);
    mem::swap(&mut new_data_pixels, &mut data_pixels);
    println!("[SYNC] Loaded new data into the task!");
}


impl Task {
    pub fn start_synchronizing(&mut self) -> JoinHandle<()> {
        let data_pixels_arc: Arc<Mutex<Vec<Pixel>>> = Arc::clone(&self.data_pixels);
        let map_arc: Arc<Mutex<HeapMap>> = Arc::clone(&self.map);
        thread::spawn(move || {
            println!("[SYNC] Sync thread made");
            thread::sleep(Duration::from_millis(100));
            loop {
                download_new_file(&data_pixels_arc);
                synchronize(&data_pixels_arc, &map_arc);
                let now = Local::now();

                let seconds = now.num_seconds_from_midnight() as u64;
                let sync_every = RESYNC_EVERY.as_secs();
                let rand_offset = RAND_OFFSET.as_secs_f64();

                let new_seconds =
                    ((seconds + sync_every + rand_offset.ceil() as u64) / sync_every) * sync_every;
                let seconds_needed = Duration::from_secs(new_seconds - seconds);
                let random_offset =
                    Duration::from_secs_f64(rand::random::<f64>() * rand_offset / 2f64);

                let time_sleeping = if rand::random::<bool>() == true {
                    seconds_needed + random_offset
                } else {
                    seconds_needed - random_offset
                };
                let done_at = now + time_sleeping;
                println!(
                    "[SYNC] I will resync again at: {}",
                    done_at.format("%H:%M:%S")
                );
                thread::sleep(time_sleeping);
            }
        })
    }
}
