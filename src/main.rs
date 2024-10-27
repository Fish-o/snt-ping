// CHANGE THESE TO LIMIT NETWORK USAGE
const SLEEP_PER_CYCLE: Option<Duration> = Some(Duration::from_millis(100));
const SLEEP_PER_PIXEL: Option<Duration> = None; // Only change this if SLEEP_PER_CYCLE isn't enough.


use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv6;
use pnet::{
    packet::{
        icmpv6::{Icmpv6Types, MutableIcmpv6Packet},
        ip::IpNextHeaderProtocols,
        Packet,
    },
    transport::{transport_channel, TransportSender},
    util,
};
use std::sync::{Arc, Mutex};

use std::net::IpAddr;
use std::str::FromStr;
use std::thread::{self, sleep};
use std::time::Duration;
mod synchronize;

#[derive(Debug, Clone, Copy)]
pub struct Pixel(u16, u16);
impl Pixel {
    pub fn new(x: usize, y: usize) -> Self {
        debug_assert!(x < 1920, "X should be within 0..1920");
        debug_assert!(y < 1080, "Y should be within 0..1080");
        Self(x as u16, y as u16)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Color(u8, u8, u8);
impl Color {
    pub fn new(r: &usize, g: &usize, b: &usize) -> Self {
        debug_assert!(r < &256, "R not 0..256");
        debug_assert!(g < &256, "G not 0..256");
        debug_assert!(b < &256, "B not 0..256");
        Self(*r as u8, *g as u8, *b as u8)
    }
    pub fn from_hex(hex: &str) -> Self {
        let data = usize::from_str_radix(hex, 16).expect("Invalid hex color entered");
        let r = (data & 0xff0000) >> (2 * 8);
        let g = (data & 0x00ff00) >> (1 * 8);
        let b = (data & 0x0000ff) >> (0 * 8);
        Self::new(&r, &g, &b)
    }
    pub fn test() {
        println!("Color  R: {:?}", Color::from_hex("FF0000"));
        println!("Color  G: {:?}", Color::from_hex("00FF00"));
        println!("Color  B: {:?}", Color::from_hex("0000FF"));
        println!("Color  1: {:?}", Color::from_hex("010101"));
        println!("Color 17: {:?}", Color::from_hex("111111"));
    }
}
pub type PixelMap = [[Option<Color>; 1080]; 1920];
pub type HeapMap = Box<PixelMap>;

pub fn create_empty_map_on_heap() -> HeapMap {
    use std::alloc::{alloc, dealloc, Layout};
    // Box::new([[Option::None; 1080]; 1920]);
    unsafe {
        let layout = Layout::new::<PixelMap>();
        let ptr = alloc(layout) as *mut PixelMap;
        Box::from_raw(ptr)
    }
}

#[derive(Clone)]
struct Task {
    map: Arc<Mutex<HeapMap>>,
    data_pixels: Arc<Mutex<Vec<Pixel>>>,
}
impl Task {
    pub fn blank() -> Self {
        Self {
            map: Arc::new(Mutex::new(create_empty_map_on_heap())),
            data_pixels: Arc::new(Mutex::new(vec![])),
        }
    }
    fn get_colored_pixel(&self, p: &Pixel) -> Color {
        let map = self.map.lock().expect("Could not aquire mutex");
        map[p.0 as usize][p.1 as usize].expect("Data pixel must contain a color")
    }
    fn print_once(&self, tx: &mut TransportSender) -> (u64, usize) {
        let mut time_wasted: u64 = 0;
        let data_pixels = self.data_pixels.lock().unwrap();
        let len = data_pixels.len();
        for p in data_pixels.iter() {
            match SLEEP_PER_PIXEL {
                Some(dur) => thread::sleep(dur),
                _ => {}
            }
            write_pixel(tx, p, &self.get_colored_pixel(p), 1, &mut time_wasted);
        }
        (time_wasted, len)
    }
}

fn main() -> Result<(), std::io::Error> {
    println!("[MAIN] Hello! Cool to see you helping out for this rightous cause!");
    println!("[MAIN] Starting up...");
    let mut task = Task::blank();
    println!("[MAIN] Starting the sync thread...");
    let t = task.start_synchronizing();
    println!("[NET]  Creating IPV6 socket");
    let mut txv6 = create_tx();
    loop {
        let now = std::time::Instant::now();
        let (wasted_time, pixels) = task.print_once(&mut txv6);
        if pixels == 0 {
            println!("[NET]  No pixels yet, waiting for data..");
            thread::sleep(Duration::from_millis(500));
            continue;
        }
        let elapsed = now.elapsed();
        let bandwidth = (pixels * 70 * 8) as f64 / elapsed.as_secs_f64();
        let mbps = bandwidth / 1000_000f64;
        let avg_mbps = match SLEEP_PER_CYCLE {
            Some(dur) =>format!("(Avg: {:.1} Mbps)",  ((pixels * 70 * 8) as f64 / (elapsed+dur).as_secs_f64())/1000_000f64),
            _=>format!("")
        };
        let pixels = if pixels >= 1000 {
            format!("{}k", pixels/1000)
        } else{
            format!("{pixels}")
        };
        println!(
            "[NET]  One full write done in {:.1?}   Net bottleneck: {:.0}%   Pixels: {pixels}   {mbps:.1} Mbps {avg_mbps}",
            elapsed,
             (wasted_time as f64 / elapsed.as_millis() as f64) * 100f64 
        );
        match SLEEP_PER_CYCLE {
            Some(dur) => thread::sleep(dur),
            _ => {}
        }
    }

    println!("[MAIN] Waiting for sync thead to join.");
    t.join().expect("Joining sync_thread failed");
    println!("[MAIN] Exiting");
    Ok(())
}

const SOCKET_ERR: &str = "Run this program with root permissions";
fn create_tx() -> TransportSender {
    let protocolv6 = Layer4(Ipv6(IpNextHeaderProtocols::Icmpv6));

    let (txv6, _) = transport_channel(4096, protocolv6).expect(&format!(
        "Could not create IPv6 socket. \n\n\n      {}\n====> {SOCKET_ERR} <====\n      {}\nIt needs root permissions to create a raw IPV6 socket, it wont hack you pinky promise <3\n\n",
        "▼".repeat(SOCKET_ERR.len()),
        "▲".repeat(SOCKET_ERR.len())
    ));
    return txv6;
}
fn write_pixel(
    tx: &mut TransportSender,
    p: &Pixel,
    c: &Color,
    attempt: u32,
    time_wasted: &mut u64,
) {
    let x = p.0;
    let y = p.1;
    let r = c.0;
    let g = c.1;
    let b = c.2;
    let a = 255;
    // 2001:610:1908:a000: <X>  : <Y>  : <B><G> : <R><A>
    let ip_str = format!("2001:610:1908:a000:{x:04x}:{y:04x}:{b:02x}{g:02x}:{r:02x}{a:02x}");
    let ipv6 = IpAddr::from_str(&ip_str).unwrap();
    match send_echov6(tx, ipv6) {
        Err(_) => {
            let t = 1u64.pow(attempt);
            // println!("{x},{y} Faild attempt #{attempt}, sleeping ({t} ms), reason for failure: {e}");
            sleep(Duration::from_millis(t));
            *time_wasted += t;
            write_pixel(tx, p, c, attempt + 1, time_wasted);
        }
        _ => {}
    }
}

fn send_echov6(tx: &mut TransportSender, addr: IpAddr) -> Result<usize, std::io::Error> {
    // Allocate enough space for a new packet
    let mut vec: Vec<u8> = vec![0; 16];

    // Use echo_request so we can set the identifier and sequence number
    let mut echo_packet = MutableIcmpv6Packet::new(&mut vec[..]).unwrap();
    echo_packet.set_icmpv6_type(Icmpv6Types::EchoRequest);

    let csum = icmpv6_checksum(&echo_packet);
    echo_packet.set_checksum(csum);

    tx.send_to(echo_packet, addr)
}

fn icmpv6_checksum(packet: &MutableIcmpv6Packet) -> u16 {
    util::checksum(packet.packet(), 1)
}
