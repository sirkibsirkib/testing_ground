#[macro_use]
extern crate lazy_static;
extern crate array_init;
extern crate rand;
extern crate noise;

mod asciireen;
use asciireen::Asciireen;
mod procedural;
mod world;
use world::World;

pub type Point = [f32;2];

use std::path::Path;

const WIDTH : u8 = LENGTH * 2;
const LENGTH : u8 = 60;
// const SYMBOLS : &'static [char] = &[' ','`','.', '-', ':',     '+','=','%',  '@'];

fn sigmoid(x : f32, amplifier : f32) -> f32 {
    let o = 1.0 /
    (1.0 + ::std::f32::consts::E.powf(x*amplifier));
    2.0 * (o - 0.5)
}

fn main() {
    // let mut a = Asciireen::new(WIDTH, LENGTH);
    for i in 0..15 {
        let w = World::new(rand::random());
        let x = w.to_png(Path::new(&format!("./map_{}.png", i)), 200);
    }
    // println!("print went {:?}", x);
    // for x in 0..WIDTH {
    //     for y in 0..LENGTH {
    //         let pt = [x as f32 / (WIDTH as f32), y as f32 / (LENGTH as f32)];
    //         a.set(x, y, sample_to_u8(w.sample(pt)));
    //     }
    // }
    // a.print_func(|x| SYMBOLS[x as usize])
}

// fn sample_to_u8(sample : f32) -> u8 {
//      (
//          (1.0 + sample) * (SYMBOLS.len() as f32) / 2.0
//      ) as u8
// }
