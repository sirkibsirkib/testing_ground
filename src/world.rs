use super::procedural::NoiseField;
use super::{Point};
use ::rand::{SeedableRng,Rng,Isaac64Rng};
use std::path::Path;
use super::sigmoid;

extern crate image;

pub struct World {
    nf : NoiseField,
    Water_level : f32,
    pole_heights : [f32;2],
    temp_equator : f32,     //0==freezing, 1==boiling
    temp_pole : f32,
}

const AZIMUTH_SHIFT : f32 = 0.003;
const AZIMUTH_MULT : f32 = 1.0 - AZIMUTH_SHIFT;

type FloatPixel = [f32 ; 3];
type U8Pixel = [u8 ; 3];

#[derive(PartialEq,Eq,Copy,Clone)]
pub enum Material {
    Rock, Foliage, Water, Ice, Snow,
}

impl Material {
    fn col(&self) -> FloatPixel {
        match self {
            &Material::Rock => [0.7, 0.4, 0.25],
            &Material::Foliage => [0.3, 0.6, 0.4],
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

impl World {
    pub fn new(seed : u64) -> World {
        let mut rng = Isaac64Rng::from_seed(&[seed]);
        let nf = NoiseField::from_super_seed(rng.gen());
        let pole_heights = [
            Self::raw_sample(&nf, [rng.gen(), 0.0]),
            Self::raw_sample(&nf, [rng.gen(), 1.0]),
        ];
        let temps : [f32;2] = ::array_init::array_init(|_| rng.gen::<f32>() * 15.0 - 5.0);
        World {
            nf : nf,
            Water_level : (0.9*rng.gen::<f32>()).powf(1.4),
            pole_heights : pole_heights,
            temp_equator : if temps[0] > temps[1] {temps[0]} else {temps[1]},
            temp_pole :  if temps[0] < temps[1] {temps[0]} else {temps[1]},
        }
    }

    fn dist_to_pole(y : f32) -> f32 {
        assert!(y <= 1.0 && y >= 0.0);
        if y < 0.5 {y} else {1.0-y}
    }

    // (-1.0, 1.0)
    pub fn sample(&self, pt : Point) -> f32 {
        let raw_noise = Self::raw_sample(&self.nf, pt);
        if pt[1] < 0.5 {
            let pole_influence = ((0.5 - pt[1]) * 2.0).powf(1.5);
            (raw_noise * (1.0 - pole_influence)) + (self.pole_heights[0] * pole_influence)
        } else {
            let pole_influence = ((pt[1] - 0.5) * 2.0).powf(1.5);
            (raw_noise * (1.0 - pole_influence)) + (self.pole_heights[1] * pole_influence)
        }
    }

    fn temp_at(&self, pt : Point) -> f32 {
        let d = Self::dist_to_pole(pt[1]);
        let h_temp = self.temp_equator * (2.0 * d) + self.temp_pole * (2.0 * (0.5-d));
        (1.0 - (self.height_at(pt) + self.slope_at(pt).abs()) * 0.5) * h_temp
    }

    // (-1.0, 1.0)
    fn raw_sample(nf : &NoiseField, pt : Point) -> f32 {
        let zoom = 0.08;
        let x1 = pt[0] / 2.0;
        let x2 = x1 + 0.5;
        let y = pt[1] / 2.0;
        nf.sample([1.4*x1 / zoom, y / zoom]) * pwr(x1)
        + nf.sample([1.4*x2 / zoom, y / zoom]) * pwr(x2)
    }

    pub fn material_at(&self, pt : Point) -> Material {
        let temp = self.temp_at(pt);
        if self.height_at(pt) < self.Water_level {
            if temp < 0.0 {Material::Ice}
            else if temp > 1.0 {Material::Rock}
            else {Material::Water}
        } else {
            if temp > 0.6 {Material::Rock}
            else if temp < 0.0 {Material::Ice}
            else {Material::Foliage}
        }
    }


    // (-1.0, 1.0)
    fn slope_at(&self, pt : Point) -> f32 {
        let pt = [pt[0]*AZIMUTH_MULT, pt[1]*AZIMUTH_MULT];
        sigmoid(
            (self.sample(pt) - self.sample([pt[0]+AZIMUTH_SHIFT, pt[1]+AZIMUTH_SHIFT])) * 0.5,
            60.0,
        )
    }

    // (0.0, 1.0)
    fn height_at(&self, pt : Point) -> f32 {
        self.sample(pt) * 0.5 + 0.5
    }

    fn pixel_sample(&self, pt : Point) -> U8Pixel {
        let height = self.height_at(pt);
        // let pole_dist = 1.0 - (0.5 - pt[1]).abs() * 2.0;
        px_finalize(
            {
                let mat = self.material_at(pt);
                if mat == Material::Water {
                    px_shade(mat.col(), 0.7 + (height * 0.3))
                } else {
                    px_shade(mat.col(), 0.8 + (self.slope_at(pt) * 0.2))
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
