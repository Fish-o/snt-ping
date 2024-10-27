use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy)]
pub struct Pixel(pub u16, pub u16);
impl Pixel {
    pub fn new(x: usize, y: usize) -> Self {
        debug_assert!(x < 1920, "X should be within 0..1920");
        debug_assert!(y < 1080, "Y should be within 0..1080");
        Self(x as u16, y as u16)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);
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
    // I dont want the program to allocate all that pixel data on the stack
    // That'd kinda cause a stack overflow. Hence this workaround to only use
    // the heap.
    use std::alloc::{alloc, Layout};
    unsafe {
        let layout = Layout::new::<PixelMap>();
        let ptr = alloc(layout) as *mut PixelMap;
        Box::from_raw(ptr)
    }
}

#[derive(Clone)]
pub struct Task {
    pub map: Arc<Mutex<HeapMap>>,
    pub data_pixels: Arc<Mutex<Vec<Pixel>>>,
}

impl Task {
    pub fn blank() -> Self {
        Self {
            map: Arc::new(Mutex::new(create_empty_map_on_heap())),
            data_pixels: Arc::new(Mutex::new(vec![])),
        }
    }
    pub fn get_colored_pixel(&self, p: &Pixel) -> Color {
        let map = self.map.lock().expect("Could not aquire mutex");
        map[p.0 as usize][p.1 as usize].expect("Data pixel must contain a color")
    }
}
