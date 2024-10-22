/*

    All values are in hexadecimal notation. And the resolution of the screen is 1920x1080 pixels.

    Example: to make the pixel at (25,25) SNT Yellow (#FFD100) with 100% opacity, execute the following command:
          2001:610:1908:a000: <X>  : <Y>  : <B><G> : <R><A>
    ping6 2001:610:1908:a000: 0019 : 0019 : 00d1   : ffff
                              16 bits
    0x19 -> 16 + 9 = 25

    1 2 3 4  5  6  7   8   9   10   11
    2 4 8 16 32 64 128 256 512 1024 2028
    64 bits

    0(+11)      11(+11)      22(+8)   30(+8)   38(+8)   46(+18)             64
    XXXXXXXXXXX YYYYYYYYYYY  RRRRRRRR GGGGGGGG BBBBBBBB OOOOOOOOOOOOOOOOOO
                                                        ^^ Compleet ongebruikte data!
*/

use image::{DynamicImage, GenericImageView, ImageReader, Pixels, Rgba};
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
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::Write;
use std::net::IpAddr;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
struct Pixel(u16, u16);
impl Pixel {
    pub fn new(x: usize, y: usize) -> Self {
        debug_assert!(x < 1920, "X should be within 0..1920");
        debug_assert!(y < 1080, "Y should be within 0..1080");
        Self(x as u16, y as u16)
    }
}
#[derive(Debug, Clone, Copy)]
struct Color(u8, u8, u8);
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

struct Task {
    map: Box<[[Option<Color>; 1080]; 1920]>,
    data_pixels: Vec<Pixel>,
}
impl Task {
    pub fn blank() -> Self {
        Self {
            map: Box::new([[Option::None; 1080]; 1920]),
            data_pixels: vec![],
        }
    }
    fn load_from_pixels(&mut self, p: Pixels<DynamicImage>) {
        self.data_pixels.clear();
        for (x, y, rgba) in p {
            if rgba.0[3] < 128 {
                self.map[x as usize][y as usize] = None;
                continue;
            }
            self.map[x as usize][y as usize] = Some(Color(rgba.0[0], rgba.0[1], rgba.0[2]));
            self.data_pixels.push(Pixel(x as u16, y as u16));
        }
        self.data_pixels.shuffle(&mut thread_rng());
    }
    fn get_colored_pixel(&self, p: &Pixel) -> Color {
        self.map[p.0 as usize][p.1 as usize].expect("Data pixel must contain a color")
    }
    fn print_once(&self, tx: &mut TransportSender) {
        for p in &self.data_pixels {
            write_pixel(tx, p, &self.get_colored_pixel(p), 1);
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    println!("Hello! Im going to be sending pings!");
    println!("Let me open up the image real quick...");
    let img = ImageReader::open("map.png")
        .expect("Opening the image 'map.png' failed")
        .decode()
        .expect("Decoding the image failed.");
    assert!(img.width() == 1920, "Image width must be 1920 pixels");
    assert!(img.height() == 1080, "Image height must be 1080 pixels");
    println!("Cool! The image looks good! Let me parse it into a task!");
    std::io::stdout().flush()?;
    let mut task = Task::blank();
    println!("Loading pixels....");
    task.load_from_pixels(img.pixels());
    println!(
        "Alright, done, i counted {} pixels with color data!",
        task.data_pixels.len()
    );
    println!("\nNext up: setting up a socket...");
    let mut txv6 = create_tx();
    println!("Socket made! Im ready to rock!");

    println!("\nWriting all pixels once:");
    let now = std::time::Instant::now();
    task.print_once(&mut txv6);
    let elapsed = now.elapsed();
    println!("Wow, that was a lot of work! it took: {:?}", elapsed);
    // write_pixel(&mut txv6, &Pixel::new(25, 25), &Color::from_hex("FFD100"));

    Ok(())
}

fn create_tx() -> TransportSender {
    let protocolv6 = Layer4(Ipv6(IpNextHeaderProtocols::Icmpv6));
    let (txv6, _) = transport_channel(4096, protocolv6)
        .expect("Could not create IPv6 socket. Are the permissions correct?");
    return txv6;
}
fn write_pixel(tx: &mut TransportSender, p: &Pixel, c: &Color, attempt: u32) {
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
        Err(e) => {
            let t = 1u64.pow(attempt);
            // println!("{x},{y} Faild attempt #{attempt}, sleeping ({t} ms), reason for failure: {e}");
            sleep(Duration::from_millis(t));
            write_pixel(tx, p, c, attempt + 1);
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
