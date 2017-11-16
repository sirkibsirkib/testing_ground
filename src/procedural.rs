extern crate noise;
use self::noise::{Perlin,Seedable,NoiseModule};
use super::Point;


extern crate rand;
use self::rand::{SeedableRng,Rng,Isaac64Rng};

const NUM_PERLINS : usize = 50;

lazy_static! {
    static ref PERLINS : [Perlin ; NUM_PERLINS] = {
        let p1 = Perlin::new();
        ::array_init::array_init(|x| p1.set_seed(x))
    };
}

#[derive(Debug)]
pub struct NoiseField {
    perlins1 : [&'static Perlin ; NoiseField::PERLINS],
    perlins2 : [&'static Perlin ; NoiseField::PERLINS],
    zooms : [f32 ; NoiseField::PERLINS*2],
    zoomtot : f32,
    super_seed : u64,
    pole_heights : [f32 ; 2],
}

const SIG_AMPLIFIER : f32 = 5.0;


impl NoiseField {

    const PERLINS : usize = 2;
    pub fn get_super_seed(&self) -> u64 {
        self.super_seed
    }

    pub fn from_super_seed(super_seed : u64) -> NoiseField {
        let mut rng = Isaac64Rng::from_seed(&[super_seed]);
        let zooms = ::array_init::array_init(|_| rng.next_f32().powf(1.2));
        // let zooms = [rng.next_f32() ; NoiseField::PERLINS*2];
        println!("{:?}", &zooms);
        NoiseField {
            perlins1 : ::array_init::array_init(
                |_| &PERLINS[(rng.next_u32() as usize) % NUM_PERLINS]
            ),
            perlins2 : ::array_init::array_init(
                |_| &PERLINS[(rng.next_u32() as usize) % NUM_PERLINS]
            ),
            zooms : zooms,
            zoomtot : zooms.iter().sum(),
            pole_heights : ::array_init::array_init(|_| rng.next_f32().powf(1.4)),
            super_seed : super_seed,
        }
    }

    #[inline]
    fn pt_map(pt : Point, zoom : f32) -> [f32;2] {
        [pt[0] as f32 * zoom, pt[1] as f32 * zoom]
    }

    //
    fn sigmoid(x : f32) -> f32 {
        let o = 1.0 /
        (1.0 + ::std::f32::consts::E.powf(x*SIG_AMPLIFIER));
        let o = 2.0 * (o - 0.5);
        // println!("{:?} -> {:?}", x, o);
        o
    }

    fn poleify(&self, pt_y : f32, raw_noise : f32) -> f32 {
        let pole_influence = if pt_y < 0.5 {
            ((0.5 - pt_y) * 2.0).powf(2.0)
        } else {
            ((pt_y - 0.5) * 2.0).powf(2.0)
        };
        println!("x {} pole_influence {:?}", pt_y,  pole_influence);
        (raw_noise * (1.0 - pole_influence)) + (self.pole_heights[0] * pole_influence)
    }

    pub fn sample(&self, pt : Point) -> f32 {
        let mut sample_tot : f32 = 0.0;
        for i in 0..Self::PERLINS {
            sample_tot +=
            self.perlins1[i].get(
                Self::pt_map(pt, self.zooms[i])
            )
            *
            self.perlins2[i].get(
                Self::pt_map(pt, self.zooms[i + Self::PERLINS])
            );
        };
        let x = sample_tot / (Self::PERLINS as f32);
        // print!(" {}", x);
        self.poleify(Self::sigmoid(x), pt[1])
    }
}
