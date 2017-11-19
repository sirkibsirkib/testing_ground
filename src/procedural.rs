extern crate noise;
use self::noise::{Perlin,Seedable,NoiseModule};
use super::{Point,Point3D};
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

#[derive(Debug,Clone)]
struct PerlinUnit {
    p1 : &'static Perlin,
    p2 : &'static Perlin,
    zoom1 : f32,
    zoom2 : f32,
    mult : f32,
}

#[derive(Debug,Clone)]
pub struct NoiseField {
    perlin_units : Vec<PerlinUnit>,
}

impl NoiseField {
    fn rebalance_unit_mults(perlin_units : &mut Vec<PerlinUnit>) {
        let mult_tot : f32 = perlin_units.iter().map(|x| x.mult).sum();
        for pu in perlin_units.iter_mut() {
            pu.mult = pu.mult / mult_tot;
        }
    }

    fn unbalance_unit_mults(perlin_units : &mut Vec<PerlinUnit>, desired_tot : f32) {
        assert!(desired_tot > 0.0);
        let mult_tot : f32 = perlin_units.iter().map(|x| x.mult).sum();
        for pu in perlin_units.iter_mut() {
            pu.mult = pu.mult / mult_tot * desired_tot;
        }
    }

    pub fn generate<R : Rng>(rng : &mut R, zoom_bounds : [f32;2], num_units : u8) -> NoiseField {
        assert!(zoom_bounds[0] <= zoom_bounds[1]);
        assert!(num_units > 0);
        let between = move |ratio| {
            ratio*(zoom_bounds[1] - zoom_bounds[0]) + zoom_bounds[0]
        };
        let mut perlin_units = vec![];
        for i in 0..(num_units as usize) {
            let pu = PerlinUnit{
                p1 : &PERLINS[(rng.next_u32() as usize) % NUM_PERLINS],
                p2 : &PERLINS[(rng.next_u32() as usize) % NUM_PERLINS],
                zoom1 : between(i as f32 / (if num_units == 1 {1} else {num_units-1}) as f32),
                zoom2 : between(rng.gen::<f32>() * 2.4),
                mult : rng.gen::<f32>() + 0.01,
            };
            perlin_units.push(pu);
        }
        Self::rebalance_unit_mults(&mut perlin_units);

        NoiseField {
            perlin_units : perlin_units,
        }
    }

    #[inline]
    fn pt_map(pt : Point, zoom : f32) -> Point {
        [pt[0] * zoom, pt[1] * zoom]
    }

    fn pt3d_map(pt : Point3D, zoom : f32) -> Point3D {
        [pt[0] * zoom, pt[1] * zoom, pt[2] * zoom]
    }

    pub fn agglomerate(self, other : NoiseField, relative_multipliers : Option<(f32, f32)>) -> NoiseField {
        let mut a = self.perlin_units;
        let mut b = other.perlin_units;
        if let Some((a_scale, b_scale)) = relative_multipliers {
            Self::unbalance_unit_mults(&mut a, a_scale);
            Self::unbalance_unit_mults(&mut b, b_scale);
        }
        let mut combined = a;
        combined.extend(b);
        Self::rebalance_unit_mults(&mut combined);
        NoiseField {
            perlin_units : combined,
        }
    }

    pub fn sample(&self, pt : Point) -> f32 {
        let mut sample_tot : f32 = 0.0;
        for pu in self.perlin_units.iter() {
            sample_tot +=
                pu.p1.get(Self::pt_map(pt, pu.zoom1))
                * pu.p2.get(Self::pt_map(pt, pu.zoom2))
                * pu.mult;
        }
        sigmoid(sample_tot, self.perlin_units.len() as f32)
    }

    pub fn sample_3d(&self, pt : Point3D) -> f32 {
        let mut sample_tot : f32 = 0.0;
        for pu in self.perlin_units.iter() {
            sample_tot +=
                pu.p1.get(Self::pt3d_map(pt, pu.zoom1))
                * pu.p2.get(Self::pt3d_map(pt, pu.zoom2))
                * pu.mult;
        }
        sigmoid(sample_tot, self.perlin_units.len() as f32)
    }
}
