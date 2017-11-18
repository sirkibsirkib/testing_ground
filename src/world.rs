use super::procedural::NoiseField;
use super::{Point};
use ::rand::{SeedableRng,Rng,Isaac64Rng};
use std::path::Path;
use super::{sigmoid,sig_0_pt5};

extern crate image;

struct CurveFunc {
    articulation_points : Vec<f32>,
}

impl CurveFunc {
    pub fn new(rng : &mut Isaac64Rng, num_points : u8) -> CurveFunc {
        let mut points = vec![];
        points.push(0.0);
        points.push(1.0);
        for _ in 0..num_points {
            points.push(rng.gen::<f32>());
        }
        points.sort_by(|x, y| x.partial_cmp(y).unwrap());
        CurveFunc {
            articulation_points : points,
        }
    }

    pub fn get(&self, x : f32) -> f32 {
        let mut val = 0.0;
        let mut dist = 9999.0;
        for a in self.articulation_points.iter() {
            let d = (val - *a).abs();
            if d < dist {
                dist = d;
                val = *a;
            }
        }
        val
    }
}

const AZIMUTH_SHIFT : f32 = 0.006;
const AZIMUTH_MULT : f32 = 1.0 - AZIMUTH_SHIFT;

type FloatPixel = [f32 ; 3];
type U8Pixel = [u8 ; 3];

#[derive(PartialEq,Eq,Copy,Clone)]
pub enum Material {
    Rock, Trees, Grass, Water, Ice, Snow, DarkRock,
}

impl Material {
    fn col(&self) -> FloatPixel {
        match self {
            &Material::Rock => [0.7, 0.4, 0.25],
            &Material::DarkRock => [0.65, 0.25, 0.1],
            &Material::Trees => [0.3, 0.6, 0.4],
            &Material::Grass => [0.5, 0.65, 0.4],
            &Material::Water => [0.3, 0.5, 1.0],
            &Material::Ice => [0.8, 0.8, 1.0],
            &Material::Snow => [1.0, 1.0, 1.0],
        }
    }
}


fn px_shade(x : FloatPixel, shading : f32) -> FloatPixel {
    [
        x[0] * (1.0 - shading),
        x[1] * (1.0 - shading),
        x[2] * (1.0 - shading),
    ]
}

fn px_finalize(x : FloatPixel) -> U8Pixel {
    [(x[0] * 254.0) as u8,
    (x[1] * 254.0) as u8,
    (x[2] * 254.0) as u8]
}

#[derive(Debug)]
pub struct World {
    base_height : NoiseField,
    complex_height : NoiseField,
    temp_nf : NoiseField,
    water_level : f32,
    pole_heights : [f32;2],
    snow_below_temp : f32,
    grass_within : [f32;2],
    trees_within : [f32;2],
}

fn globe_sample(nf : &NoiseField, pt : Point) -> f32 {
    let x1 = pt[0] / 2.0;
    let x2 = x1 + 0.5;
    let y = pt[1] / 2.0;
    nf.sample([1.9*x1, y]) * pwr(x1)
    + nf.sample([1.9*x2, y]) * pwr(x2)
}

#[derive(Copy,Clone,Debug)]
pub struct WorldPrimitive {
    super_seed : u64,
    distance_to_star : f32,
    star_energy : f32,
}

impl WorldPrimitive {
    pub fn new(super_seed : u64, distance_to_star : f32, star_energy : f32) -> WorldPrimitive {
        WorldPrimitive {
            super_seed : super_seed,
            distance_to_star : distance_to_star,
            star_energy : star_energy,
        }
    }
}

enum Weighting {
    Equal, Higher(f32), Lower(f32),
}

impl World {
    fn gen_between<R:Rng>(lower : f32, upper : f32, w : Weighting, rng : &mut R) -> f32 {
        let betweenyness = match w {
            self::Weighting::Equal => {
                rng.gen::<f32>()
            },
            self::Weighting::Lower(amp) => {
                1.0 - sigmoid(rng.gen::<f32>(), amp)
            },
            self::Weighting::Higher(amp) => {
                sigmoid(rng.gen::<f32>(), amp)
            },
        };
        lower * betweenyness
        + upper * (1.0 - betweenyness)
    }

    pub fn new(wp : WorldPrimitive) -> World {
        let mut rng = Isaac64Rng::from_seed(&[wp.super_seed]);

        let size = rng.gen::<f32>() * (0.2 + 0.8*wp.distance_to_star);
        let radiated_heat = wp.star_energy * (1.0 - wp.distance_to_star);
        let water_level = sig_0_pt5(rng.gen::<f32>() * size * (1.0 - radiated_heat), 0.5);


        let base_height_bounds = [
            Self::gen_between(0.05, 0.5, Weighting::Lower(3.0), &mut rng),
            Self::gen_between(0.5, 5.0, Weighting::Lower(3.0), &mut rng),
        ];

        let complex_height_bounds = [
            Self::gen_between(0.5, 1.5, Weighting::Lower(3.0), &mut rng),
            Self::gen_between(1.5, 30.0, Weighting::Lower(2.0), &mut rng),
        ];

        let base_height = NoiseField::generate(&mut rng, base_height_bounds, 5)
        .agglomerate(
            NoiseField::generate(&mut rng, [40.2, 90.4], 3), Some((1.0, 0.013))
        ).agglomerate(
            NoiseField::generate(&mut rng, [2.2, 9.4], 3), Some((1.0, 0.3))
        );

        let complex_height = NoiseField::generate(&mut rng, complex_height_bounds, 5)
        .agglomerate(
            NoiseField::generate(&mut rng, [20.2, 30.4], 2), Some((1.0, 0.12))
        );

        let pole_heights = [
            Self::raw_sample(&base_height, [rng.gen(), 0.0]) * 0.5 + 0.5,
            Self::raw_sample(&base_height, [rng.gen(), 1.0]) * 0.5 + 0.5,
        ];

        // let temps : [f32;2] = ::array_init::array_init(|_| rng.gen::<f32>() * 5.0 - 2.0);
        let w = World {
            base_height : base_height,
            complex_height : complex_height,
            temp_nf : NoiseField::generate(&mut rng, [60.0, 100.0], 3),
            water_level : water_level,
            pole_heights : pole_heights,
            snow_below_temp : -sigmoid(-radiated_heat, 4.13),
            grass_within : [0.1,0.2],
            trees_within : [0.1,0.3],
        };
        println!("w {:#?}", &w);
        w
    }

    // (-1.0, 1.0)
    fn raw_sample(nf : &NoiseField, pt : Point) -> f32 {
        globe_sample(nf, pt)
    }

    fn temp_at(&self, pt : Point) -> f32 {
        let x = self.polarize_sample(globe_sample(&self.temp_nf, pt), pt[1]) * 0.5 + 0.5;
        x * 0.15
        + (1.0-self.height_at(pt)) * 0.85
    }

    pub fn material_at(&self, pt : Point) -> Material {
        let temp = self.temp_at(pt);
        let height = self.height_at(pt);
        let slope = self.x_slope_at(pt).abs() + self.y_slope_at(pt).abs();
        let veg_dist = (((temp - 0.3).abs() + 0.01) * (slope*20.0 + height) - self.snow_below_temp).abs();
        // println!("{}, {}, {}", temp, height, slope);
        if height < self.water_level {
            if temp + 0.02 < self.snow_below_temp {Material::Ice}
            else {Material::Water}
        }
        else if temp < self.snow_below_temp {Material::Snow}
        else if slope > 0.1 {Material::DarkRock}
        else if veg_dist  < 0.02 {Material::Grass}
        else {Material::Rock}
    }


    // (-1.0, 1.0)
    fn x_slope_at(&self, pt : Point) -> f32 {
        let pt = [pt[0]*AZIMUTH_MULT, pt[1]];
        sigmoid(
            (self.height_at(pt) - self.height_at([pt[0]+AZIMUTH_SHIFT, pt[1]])) * 0.5,
            60.0,
        )
    }

    fn y_slope_at(&self, pt : Point) -> f32 {
        let pt = [pt[0], pt[1]*AZIMUTH_MULT];
        sigmoid(
            (self.height_at(pt) - self.height_at([pt[0], pt[1]+AZIMUTH_SHIFT])) * 0.5,
            60.0,
        )
    }

    // (0.0, 1.0)
    fn height_at(&self, pt : Point) -> f32 {
        let rough_sample = (globe_sample(&self.base_height, pt) * 0.5 + 0.5).powf(1.55);
        let non_polar_solution = if rough_sample > self.water_level {
            let fine_sample = globe_sample(&self.complex_height, pt) * 0.5 + 0.5;
            let fineness = rough_sample - self.water_level;
            fineness * fine_sample + (1.0 - fineness) * rough_sample
        } else {
            rough_sample
        };
        self.polarize_sample(non_polar_solution, pt[1])
    }

    fn polarize_sample(&self, sample : f32, y : f32) -> f32 {
        assert!(y <= 1.0 && y >= 0.0);
        let dist_to_pole = if y < 0.5 {y} else {1.0-y};
        let pole_weight = 1.0 - dist_to_pole*2.0;
        pole_weight * (if y<0.5 {self.pole_heights[0]} else {self.pole_heights[1]})
        + (1.0 - pole_weight) * sample
    }

    fn pixel_sample(&self, pt : Point) -> U8Pixel {
        let height = self.height_at(pt);
        px_finalize(
            {
                let mat = self.material_at(pt);
                // mat.col()
                if mat == Material::Water || mat == Material::Ice {
                    px_shade(mat.col(), 0.3 + (height * 0.7))
                } else {
                    px_shade(mat.col(), 0.2 + ((self.x_slope_at(pt)*0.5 + 0.5) * 0.8))
                }
            }
        )
    }


    pub fn to_png(&self, path : &Path, pix_height : u32) -> Result<(), ::std::io::Error>{
        let pix_width = pix_height * 2;
        let (f_width, f_height) = (pix_width as f32, pix_height as f32);
        let mut pixels : Vec<u8> = vec![];
        for y in 0..pix_height {
            for x in 0..pix_width {
                for u_eight in self.pixel_sample([x as f32 / f_width, y as f32 / f_height]).into_iter() {
                    pixels.push(*u_eight);
                }
                pixels.push(255); //a
            }
        }
        image::save_buffer(path, &pixels[..], pix_width, pix_height, image::RGBA(8))
    }
}

fn pwr(x : f32) -> f32 {
    assert!(x >= 0.0 && x <= 1.0);
    let out = 1.0 - 2.0 * (x - 0.5).abs();
    // println!("{} -> {}", x, out);
    out
}
