#[macro_use]
extern crate lazy_static;
extern crate array_init;
extern crate rand;
extern crate noise;

mod asciireen;
use asciireen::Asciireen;
mod procedural;
mod world;
use world::{WorldPrimitive,World};

use self::rand::{SeedableRng,Rng,Isaac64Rng};
pub type Point = [f32;2];

use std::path::Path;

const WIDTH : u8 = LENGTH * 2;
const LENGTH : u8 = 60;
// const SYMBOLS : &'static [char] = &[' ','`','.', '-', ':',     '+','=','%',  '@'];

fn sigmoid(x : f32, amplifier : f32) -> f32 {
    let o = 1.0 /
    (1.0 + ::std::f32::consts::E.powf(-x*amplifier));
    2.0 * (o - 0.5)
}

fn sig_0_pt5(x : f32, amplifier : f32) -> f32 {
    sigmoid(x * 2.0 - 1.0, amplifier) * 0.5 + 0.5
}

use std::thread;

const MEGA : u64 = 10;

fn does(range : [u64;2]) ->  std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut rng = Isaac64Rng::from_seed(&[range[0], MEGA]);
        for i in range[0]..range[1] {
            let wp = WorldPrimitive::new(rng.gen(), rng.gen(), rng.gen());
            let w = World::new(wp);
            let x = w.to_png(Path::new(&format!("./map_{}.png", i)), 300);
            println!("{} : {:?}", i, x);
        }
    })
}

fn main() {
    let seed = 2;
    let mut handles = vec![
        does([0, 5]),
        does([5, 10]),
        does([10, 15]),
        does([15, 20]),
    ];
    for x in handles.drain(..) {
        x.join().is_ok();
    }
}
