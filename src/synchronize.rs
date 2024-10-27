use crate::{create_empty_map_on_heap, Color, HeapMap, Pixel, Task};
use core::time;
use image::codecs::png::PngDecoder;
use image::EncodableLayout;
use image::{DynamicImage, GenericImageView, ImageReader, Pixels, Rgba};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::File;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, mem, thread};

pub fn download_new_file() {
    let resp = reqwest::blocking::get("https://github.conner.zip/map.png").expect("request failed");
    let body = resp.bytes().expect("body invalid");
    let mut out = File::create("download.png").expect("failed to create file");
    io::copy(&mut body.as_bytes(), &mut out).expect("failed to copy content");
}

impl Task {
    pub fn start_synchronizing(&mut self) -> JoinHandle<()> {
        println!("[MAIN] Copying self");
        let data_pixels_arc = Arc::clone(&self.data_pixels);
        let map_arc = Arc::clone(&self.map);
        println!("[MAIN] Creating sync thread");
        thread::spawn(move || {
            println!("[SYNC] Start synchronizing");
            thread::sleep(Duration::from_millis(1000));
            download_new_file();
            println!("[SYNC] Downloaded file");
            let img = ImageReader::open("download.png")
                .expect("Opening the image 'map.png' failed")
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
            println!("[SYNC] Swapped data!");
        })
    }
}
