use super::procedural::NoiseField;
use super::{Point,Point3D};
use ::rand::{SeedableRng,Rng,Isaac64Rng};
use std::path::Path;
use super::{sigmoid,sig_0_pt5};

extern crate image;


const AZIMUTH_SHIFT : f32 = 0.006;
const AZIMUTH_MULT : f32 = 1.0 - AZIMUTH_SHIFT;

type FloatPixel = [f32 ; 3];
type U8Pixel = [u8 ; 3];

struct PointRawNoise {
    base : f32,
    complex : f32,
    temp : f32,
}

struct PointSampleData {
    temp : f32,
    height : f32,
    x_slope : f32,
    y_slope : f32,
    slope : f32,
}

#[derive(PartialEq,Eq,Copy,Clone)]
pub enum Material {
    Rock, Trees, Grass, Water, Ice, Snow, DarkRock,
}

impl Material {
    fn col(&self) -> FloatPixel {
        match self {
            &Material::Rock => [0.5, 0.37, 0.24],
            &Material::DarkRock => [0.41, 0.27, 0.21],
            &Material::Trees => [0.3, 0.6, 0.4],
            &Material::Grass => [0.5, 0.65, 0.4],
            &Material::Water => [0.3, 0.5, 1.0],
            &Material::Ice => [1.0, 1.0, 1.3],
            &Material::Snow => [1.15, 1.15, 1.2],
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

fn px_bound(x : f32) -> f32 {
    if x < 0.0 {0.0}
    else if x > 1.0 {1.0}
    else {x}
}

fn px_finalize(x : FloatPixel) -> U8Pixel {
    [(px_bound(x[0]) * 254.0) as u8,
    (px_bound(x[1]) * 254.0) as u8,
    (px_bound(x[2]) * 254.0) as u8]
}

#[derive(Debug)]
pub struct World {
    base_height : NoiseField,
    complex_height : NoiseField,
    temp_nf : NoiseField,
    water_level : f32,
    // pole_heights : [f32;2],
    snow_below_temp : f32,
    grass_within : [f32;2],
    trees_within : [f32;2],
    size : f32,
}

// fn globe_sample(nf : &NoiseField, pt : Point) -> f32 {
//     let x1 = pt[0] / 2.0;
//     let x2 = x1 + 0.5;
//     let y = pt[1] / 2.0;
//     nf.sample([1.9*x1, y]) * pwr(x1)
//     + nf.sample([1.9*x2, y]) * pwr(x2)
// }

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

        let size = rng.gen::<f32>() * (0.2 + 0.8*wp.distance_to_star).powf(1.3);
        let radiated_heat = wp.star_energy * (1.0 - wp.distance_to_star);
        let water_level = sig_0_pt5(rng.gen::<f32>() - 0.5 + (size * (1.0 - radiated_heat)), 2.02);
        println!("water lvl {:?}", water_level);

        // let base_height_bounds = [
        //     Self::gen_between(0.02, 0.03, Weighting::Lower(3.0), &mut rng) / (size + 0.4),
        //     Self::gen_between(0.03, 0.04, Weighting::Lower(3.0), &mut rng) / (size + 0.4),
        // ];

        let base_height_bounds = [
            Self::gen_between(0.05, 0.15, Weighting::Lower(3.0), &mut rng),
            Self::gen_between(0.15, 3.0, Weighting::Lower(2.0), &mut rng),
        ];

        let complex_height_bounds = [
            Self::gen_between(0.5, 0.8, Weighting::Lower(3.0), &mut rng),
            Self::gen_between(3.5, 6.0, Weighting::Lower(2.0), &mut rng),
        ];

        let base_height = NoiseField::generate(&mut rng, base_height_bounds, 4)
        .agglomerate(
            NoiseField::generate(&mut rng, [30.2, 50.4], 2), Some((1.0, 0.013))
        ).agglomerate(
            NoiseField::generate(&mut rng, [2.2, 9.4], 3), Some((1.0, 0.1))
        );

        let complex_height = NoiseField::generate(&mut rng, complex_height_bounds, 4)
        .agglomerate(
            NoiseField::generate(&mut rng, [15.2, 32.4], 3), Some((1.0, 0.12))
        );

        let temp_nf = NoiseField::generate(&mut rng, [30.0, 100.0], 3).agglomerate(
            NoiseField::generate(&mut rng, [0.8, 6.0], 2), Some((1.0, 0.25))
        );

        // let pole_heights = [
        //     Self::raw_sample(&base_height, [rng.gen(), 0.0]) * 0.5 + 0.5,
        //     Self::raw_sample(&base_height, [rng.gen(), 1.0]) * 0.5 + 0.5,
        // ];

        // let temps : [f32;2] = ::array_init::array_init(|_| rng.gen::<f32>() * 5.0 - 2.0);
        let w = World {
            size : size,
            base_height : base_height,
            complex_height : complex_height,
            temp_nf : temp_nf,
            water_level : water_level,
            // pole_heights : pole_heights,
            snow_below_temp : -sigmoid(-radiated_heat, 4.13),
            grass_within : [0.1,0.2],
            trees_within : [0.1,0.3],
        };
        // println!("w {:#?}", &w);
        w
    }

    // fn raw_samples(&self, pt : Point) -> PointRawNoise {
    //     PointRawNoise {
    //         base : self.base_height.sample(pt),
    //         complex : self.complex_height.sample(pt),
    //         temp : self.temp_nf.sample(pt),
    //     }
    // }

    // (-1.0, 1.0)
    // fn raw_sample(nf : &NoiseField, pt : Point) -> f32 {
    //     globe_sample(nf, pt)
    // }

    fn calc_temp_at(&self, pt : Point, height : f32) -> f32 {
        let x = self.temp_nf.sample_3d(equirectangular(pt)) * 0.5 + 0.5;
        // let x = self.polarize_sample(globe_sample(&self.temp_nf, pt), pt[1]) * 0.5 + 0.5;
        x * 0.15
        + (1.0-height) * 0.85
        - sigmoid(self.size / (Self::pole_distance(pt[1]) + 0.01), 1.0) * 0.3
    }

    fn material_at(&self, pt : Point, point_data : &PointSampleData) -> Material {
        let veg_dist = (((point_data.temp - 0.3).abs() + 0.01) * (point_data.slope*20.0 + point_data.height) - self.snow_below_temp).abs();
        if point_data.height < self.water_level {
            if point_data.temp + 0.02 < self.snow_below_temp {Material::Ice}
            else {Material::Water}
        }
        else if point_data.temp < self.snow_below_temp {Material::Snow}
        else if point_data.slope > 0.12 {Material::DarkRock}
        else if veg_dist < 0.08*self.water_level && point_data.temp < 0.3 && point_data.slope > 0.01 {Material::Trees}
        else if veg_dist < 0.12*self.water_level {Material::Grass}
        else {Material::Rock}
    }

    // (0.0, 1.0)
    fn calc_height_at(&self, pt : Point) -> f32 {
        let rough_sample = (self.base_height.sample_3d(equirectangular(pt)) * 0.5 + 0.5).powf(1.55);
        // let rough_sample = (globe_sample(&self.base_height, pt) * 0.5 + 0.5).powf(1.55);
        if rough_sample > self.water_level {
            let fine_sample = self.complex_height.sample_3d(equirectangular(pt)) * 0.5 + 0.5;
            // let fine_sample = globe_sample(&self.complex_height, pt) * 0.5 + 0.5;
            let fineness = rough_sample - self.water_level;
            fineness * fine_sample + (1.0 - fineness) * rough_sample
        } else {
            rough_sample
        }
        // self.polarize_sample(non_polar_solution, pt[1])
    }

    fn pole_distance(y : f32) -> f32 {
        assert!(y <= 1.0 && y >= 0.0);
        if y < 0.5 {y} else {1.0-y}
    }

    // fn polarize_sample(&self, sample : f32, y : f32) -> f32 {
    //     let dist_to_pole = Self::pole_distance(y).powf(0.8);
    //     let pole_weight = 1.0 - dist_to_pole*2.0;
    //     pole_weight * (if y<0.5 {self.pole_heights[0]} else {self.pole_heights[1]})
    //     + (1.0 - pole_weight) * sample
    // }

    fn calc_sample_data_at(&self, pt : Point) -> PointSampleData {
        let height = self.calc_height_at(pt);
        let x_slope = {
            sigmoid(
                (height - self.calc_height_at([(pt[0]+AZIMUTH_SHIFT % 1.0), pt[1]])) * 0.5,
                30.0,
            )
        };
        let y_slope = {
            sigmoid(
                if pt[1] < AZIMUTH_MULT {
                    (height - self.calc_height_at([pt[0], pt[1]+AZIMUTH_SHIFT])) * 0.5
                } else {
                    (self.calc_height_at([pt[0], pt[1]-AZIMUTH_SHIFT]) - height) * 0.5
                },
                30.0,
            )
        };
        let slope = (x_slope.abs() + y_slope.abs()) * 0.5;
        let temp = self.calc_temp_at(pt, height);
        PointSampleData {
            height : height,
            x_slope : x_slope,
            y_slope : y_slope,
            slope : slope,
            temp : temp,
        }
    }

    fn pixel_sample(&self, pt : Point) -> U8Pixel {
        let point_data = self.calc_sample_data_at(pt);
        px_finalize(
            {
                let mat = self.material_at(pt, &point_data);
                // mat.col()
                if mat == Material::Water || mat == Material::Ice {
                    px_shade(mat.col(), 0.25 + ((1.0-point_data.height) * 0.4))
                } else {
                    px_shade(mat.col(), 0.25 + ((point_data.x_slope*0.5 + 0.5) * 0.4))
                }
            }
        )
    }


    pub fn to_png(&self, path : &Path, pix_height : u32) -> Result<(), ::std::io::Error>{
        let pix_width = pix_height * 2;
        let (f_width, f_height) = (pix_width as f32, pix_height as f32);
        let mut pixels : Vec<u8> = vec![];
        let time_0 = ::std::time::Instant::now();
        for y in 0..pix_height {
            for x in 0..pix_width {
                for u_eight in self.pixel_sample([x as f32 / f_width, y as f32 / f_height]).into_iter() {
                    pixels.push(*u_eight);
                }
                pixels.push(255); //a
            }
        }
        let time_1 = ::std::time::Instant::now();
        let res = image::save_buffer(path, &pixels[..], pix_width, pix_height, image::RGBA(8));
        let time_2 = ::std::time::Instant::now();
        let (dur_0, dur_1) = (time_1-time_0, time_2-time_1);
        // println!("{:?}/{:?} ratio: ({:?})", dur_0, dur_1, (dur_0.as_secs() as f32 / (dur_0+dur_1).as_secs() as f32));
        res
    }
}

// fn pwr(x : f32) -> f32 {
//     assert!(x >= 0.0 && x <= 1.0);
//     let out = 1.0 - 2.0 * (x - 0.5).abs();
//     // println!("{} -> {}", x, out);
//     out
// }
pub fn equirectangular(pt : Point) -> Point3D {
    let q = 1.0 - (pt[1]*2.0 - 1.0).abs();
    let l = pt[0] * ::std::f32::consts::PI * 2.0;
    [
        l.sin() * q,
        l.cos() * q,
        pt[1],
    ]
}
