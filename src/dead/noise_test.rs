extern crate noise;
// use self::noise::*;
use self::noise::{Seedable,NoiseModule,Perlin,Point2};
extern crate rand;
use self::rand::Rng;

pub fn nnn() {
    let map = Map::new(80,40,rand::thread_rng().gen(),20.0);
    map.draw();
}

struct Map {
    width : u32,
    height : u32,
    coarseness : f64,
    perlin1 : Perlin,
    perlin2 : Perlin,
    perlin3 : Perlin,
}



macro_rules! mmm {
    ( $( $x:expr ),* ) => {
        {
            let mut m = None;
            $(
                if let Some(sm) = m {
                    m = Some(::std::cmp::min($x, sm));
                } else {
                    m = Some($x);
                }
            )*
            m.unwrap()
        }
    };
}

impl Map {

    fn new(width : u32, height : u32, seed : usize, coarseness : f64) -> Map {
        println!("SEED {}", seed);
        let p1 = Perlin::new();
        Map {
            width : width,
            height : height,
            perlin1 : p1.set_seed(seed),
            perlin2 : p1.set_seed(seed+1),
            perlin3 : p1.set_seed(seed+2),
            coarseness : coarseness,
        }
    }

    #[inline]
    fn noise_map(x : f64) -> char {
        if x < -0.1 {
            ' '
        } else if x < 0.5 {
            '~'
        } else {
            '#'
        }
    }


    fn draw(&self) {
        for j in 0..self.height {
            let y = j as f64 / self.coarseness;
            for i in 0..self.width {
                let x = i as f64 / self.coarseness;
                let wall_dist : f64 = mmm!(i, self.width-i, j, self.height-j) as f64;
                print!("{}",
                    Self::noise_map(
                        self.perlin1.get([x, y])
                        + self.perlin2.get([x*1.8, y*1.8])*0.5
                        + self.perlin3.get([x*0.3, y*0.3])*0.5
                        + 3.0 / wall_dist
                    )
                );
            }
            println!();
        }
    }
}
