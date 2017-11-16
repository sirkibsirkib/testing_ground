#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate array_init;

extern crate rand;

pub type Point = [f32;2];

mod asciireen;
mod procedural;

use asciireen::Asciireen;
use procedural::NoiseField;

const WIDTH : u8 = 80;
const LENGTH : u8 = 40;

const SYMBOLS : &'static [char] = &['_', ',', ',', '.', '*', '^', '"', '\'', '`', ':'];

fn main() {
    let mut a = Asciireen::new(WIDTH, LENGTH);
    let nf = NoiseField::from_super_seed(rand::random());
    for x in 0..WIDTH {
        for y in 0..LENGTH {
            let pt = [x as f32 / (WIDTH as f32), y as f32 / (LENGTH as f32)];
            a.set(x, y, ((1.0 + sample_world_pt(& nf, pt)) * 5.0 ) as u8);
        }
    }
    // println!("{}", &a);
    a.print_func(|x| SYMBOLS[x as usize])
}

fn pwr(x : f32) -> f32 {
    assert!(x >= 0.0 && x <= 1.0);
    let out = 1.0 - 2.0 * (x - 0.5).abs();
    // println!("{} -> {}", x, out);
    out
}

#[inline]
fn scale(pt : Point, zoom : f32) -> Point {
     [
        pt[0] / zoom,
        pt[1] / zoom,
     ]
 }
fn sample_world_pt(nf : &NoiseField, pt : Point) -> f32 {
    let zoom = 0.15;
    let x1 = pt[0] / 2.0;
    let x2 = x1 + 0.5;
    let y = pt[1] / 2.0;
    nf.sample(scale([x1,y], zoom)) * pwr(x1)
    + nf.sample(scale([x2,y], zoom)) * pwr(x2)
}
