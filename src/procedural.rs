extern crate noise;
use self::noise::{Perlin,Seedable,NoiseModule};
use super::Point;
use super::sigmoid;


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
    meta_zoom : f32,
    //mult.iter().sum() == 1.0
    mults : [f32 ; NoiseField::PERLINS],
    super_seed : u64,
    pole_heights : [f32 ; 2],
}

impl NoiseField {

    const PERLINS : usize = 6;
    pub fn get_super_seed(&self) -> u64 {
        self.super_seed
    }

    pub fn from_super_seed(super_seed : u64, zoom : f32) -> NoiseField {
        let mut rng = Isaac64Rng::from_seed(&[super_seed]);
        let mut zooms : [f32;NoiseField::PERLINS*2] =
            ::array_init::array_init(|_| (rng.next_f32() * 1.09).powf(6.0));
        let mut mults : [f32 ; NoiseField::PERLINS] =
            ::array_init::array_init(|_| (0.1 + rng.next_f32() * 0.9));
        mults[0] = 0.02; //gentle
        zooms[0] = 6.7; //need at least one to be fine noise
        zooms[NoiseField::PERLINS] = 3.9; //need at least one to be fine noise
        // let zooms = [rng.next_f32() ; NoiseField::PERLINS*2];
        // println!("{:?}", &zooms);

        let mult_tot = mults.iter().sum();
        for m in mults.iter_mut() {
            *m /= mult_tot;
        }
        NoiseField {
            meta_zoom : zoom,
            perlins1 : ::array_init::array_init(
                |_| &PERLINS[(rng.next_u32() as usize) % NUM_PERLINS]
            ),
            perlins2 : ::array_init::array_init(
                |_| &PERLINS[(rng.next_u32() as usize) % NUM_PERLINS]
            ),
            zooms : zooms,
            mults : mults,
            // zoomtot : zooms.iter().sum(),
            pole_heights : ::array_init::array_init(|_| rng.next_f32().powf(1.2)),
            super_seed : super_seed,
        }
    }

    #[inline]
    fn pt_map(pt : Point, zoom : f32) -> [f32;2] {
        [pt[0] as f32 * zoom, pt[1] as f32 * zoom]
    }

    pub fn sample(&self, pt : Point) -> f32 {
        let mut sample_tot : f32 = 0.0;
        for i in 0..Self::PERLINS {
            sample_tot +=
            self.perlins1[i].get(
                Self::pt_map(pt, self.zooms[i] * self.meta_zoom)
            )
            *
            self.perlins2[i].get(
                Self::pt_map(pt, self.zooms[i + Self::PERLINS] * self.meta_zoom)
            )
            *
            self.mults[i];
        };
        // print!(" {}", x);
        sigmoid(sample_tot, Self::PERLINS as f32 * 2.0)
    }
}
