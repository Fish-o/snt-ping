
// You're looking at the source code! To make sure
// you do not get scared, make sure to not look below
// the "ONLY CODE BELOW" line.

const SLEEP_PER_CYCLE: Option<Duration> = Some(Duration::from_millis(100));

// Unless really needed, you should not use nops, since it busy waits.
// Uncomment the 'nops' macro, and comment the other one, to add a delay per pixel.
// You can add more nops to make it slower, or remove them to speed it up.

// macro_rules! nops {() =>{ asm!("nop","nop","nop","nop","nop","nop","nop","nop","nop","nop","nop","nop");}}
macro_rules! nops {() =>{}}


/*
    v-v-v-v-v-v-v-v-v ONLY CODE BELOW v-v-v-v-v-v-v-v-v
*/

use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv6;
use pnet::{
    packet::{
        icmpv6::{Icmpv6Types,MutableIcmpv6Packet},
        ip::IpNextHeaderProtocols,
        Packet,
    },
    transport::{transport_channel,TransportSender},
    util,
};
#[allow(unused_imports)]
use std::arch::asm;
use std::net::IpAddr;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use crate::utils::{Color,Pixel};
use crate::Task;

macro_rules! nops2 {() => {nops!();nops!();nops!();nops!();nops!();nops!();nops!();nops!();nops!();nops!();}}
macro_rules! nops3 {() => {nops2!();nops2!();nops2!();nops2!();nops2!();nops2!();nops2!();nops2!();nops2!();nops2!();}}
macro_rules! nop_sleep {() => {#[allow(unused_unsafe)] unsafe{nops3!();nops3!();nops3!();nops3!();nops3!();nops3!();nops3!();nops3!();nops3!();nops3!();}}}
  
impl Task{
    fn print_once(&self,tx: &mut TransportSender) -> (u64,usize) {
        let mut time_wasted: u64 = 0;
        let data_pixels = self.data_pixels.lock().unwrap();
        let len = data_pixels.len();
        for p in data_pixels.iter() {
            nop_sleep!();
            write_pixel(tx,p,&self.get_colored_pixel(p),1,&mut time_wasted);
        }
        (time_wasted,len)
    }
}
pub fn start_network_loop(task: &mut Task){
    println!("[NET]  Creating IPV6 socket");
    let mut txv6 = create_tx();
    loop {
        let now = std::time::Instant::now();
        let (wasted_time,pixels) = task.print_once(&mut txv6);
        if pixels == 0 {
            println!("[NET]  No pixels yet,waiting for data..");
            sleep(Duration::from_millis(500));
            continue;
        }
        let elapsed = now.elapsed();
        let bandwidth = (pixels * 70 * 8) as f64 / elapsed.as_secs_f64();
        let mbps = bandwidth / 1000_000f64;
        let avg_mbps = match SLEEP_PER_CYCLE {
            Some(dur) =>format!("(Avg: {:.1} Mbps)", ((pixels * 70 * 8) as f64 / (elapsed+dur).as_secs_f64())/1000_000f64),
            _=>format!("")
        };
        let pixels = if pixels >= 1000 {
            format!("{}k",pixels/1000)
        } else{
            format!("{pixels}")
        };
        println!(
            "[NET]  One full write done in {:.1?}   Net bottleneck: {:.0}%   Pixels: {pixels}   {mbps:.1} Mbps {avg_mbps}",
            elapsed,
             (wasted_time as f64 / elapsed.as_millis() as f64) * 100f64 
        );
        match SLEEP_PER_CYCLE {
            Some(dur) => sleep(dur),
            _ => {}
        }
    }
}


const SOCKET_ERR: &str = "Run this program with root permissions";
pub fn create_tx() -> TransportSender {
    let protocolv6 = Layer4(Ipv6(IpNextHeaderProtocols::Icmpv6));

    let (txv6,_) = transport_channel(4096,protocolv6).expect(&format!(
        "Could not create IPv6 socket. \n\n\n      {}\n====> {SOCKET_ERR} <====\n      {}\nIt needs root permissions to create a raw IPV6 socket,it wont hack you pinky promise <3\n\n",
        "▼".repeat(SOCKET_ERR.len()),
        "▲".repeat(SOCKET_ERR.len())
    ));
    return txv6;
}
pub fn write_pixel(
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
    match send_echov6(tx,ipv6) {
        Err(_) => {
            let t = 1u64.pow(attempt);
            // println!("{x},{y} Faild attempt #{attempt},sleeping ({t} ms),reason for failure: {e}");
            sleep(Duration::from_millis(t));
            *time_wasted += t;
            write_pixel(tx,p,c,attempt + 1,time_wasted);
        }
        _ => {}
    }
}

fn send_echov6(tx: &mut TransportSender,addr: IpAddr) -> Result<usize,std::io::Error> {
    // Allocate enough space for a new packet
    let mut vec: Vec<u8> = vec![0; 16];

    // Use echo_request so we can set the identifier and sequence number
    let mut echo_packet = MutableIcmpv6Packet::new(&mut vec[..]).unwrap();
    echo_packet.set_icmpv6_type(Icmpv6Types::EchoRequest);

    let csum = icmpv6_checksum(&echo_packet);
    echo_packet.set_checksum(csum);

    tx.send_to(echo_packet,addr)
}

fn icmpv6_checksum(packet: &MutableIcmpv6Packet) -> u16 {
    util::checksum(packet.packet(),1)
}
