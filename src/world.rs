use super::procedural::NoiseField;
use super::{Point};
use ::rand::{SeedableRng,Rng,Isaac64Rng};
use std::path::Path;
use super::sigmoid;

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

const AZIMUTH_SHIFT : f32 = 0.003;
const AZIMUTH_MULT : f32 = 1.0 - AZIMUTH_SHIFT;

type FloatPixel = [f32 ; 3];
type U8Pixel = [u8 ; 3];

#[derive(PartialEq,Eq,Copy,Clone)]
pub enum Material {
    Rock, Foliage, Water, Ice, Snow, HotRock,
}

impl Material {
    fn col(&self) -> FloatPixel {
        match self {
            &Material::Rock => [0.7, 0.4, 0.25],
            &Material::HotRock => [0.7, 0.2, 0.1],
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

pub struct World {
    height_nf : NoiseField,
    temp_nf : NoiseField,
    water_level : f32,
    pole_heights : [f32;2],
    temp_equator : f32,     //0==freezing, 1==boiling
    temp_pole : f32,
    height_curve : CurveFunc,
}

fn globe_sample(nf : &NoiseField, pt : Point) -> f32 {
    let x1 = pt[0] / 2.0;
    let x2 = x1 + 0.5;
    let y = pt[1] / 2.0;
    nf.sample([1.4*x1, y]) * pwr(x1)
    + nf.sample([1.4*x2, y]) * pwr(x2)
}

impl World {
    pub fn new(seed : u64) -> World {
        let mut rng = Isaac64Rng::from_seed(&[seed]);
        let height_nf = NoiseField::from_super_seed(rng.gen(), 10.0);
        let pole_heights = [
            Self::raw_sample(&height_nf, [rng.gen(), 0.0]),
            Self::raw_sample(&height_nf, [rng.gen(), 1.0]),
        ];
        let temps : [f32;2] = ::array_init::array_init(|_| rng.gen::<f32>() * 5.0 - 2.0);
        World {
            height_nf : height_nf,
            height_curve : CurveFunc::new(&mut rng, 5),
            temp_nf : NoiseField::from_super_seed(rng.gen(), 20.0),
            water_level : (0.9*rng.gen::<f32>()).powf(1.4),
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
        let raw_noise = Self::raw_sample(&self.height_nf, pt);
        if pt[1] < 0.5 {
            let pole_influence = ((0.5 - pt[1]) * 2.0).powf(1.5);
            (raw_noise * (1.0 - pole_influence)) + (self.pole_heights[0] * pole_influence)
        } else {
            let pole_influence = ((pt[1] - 0.5) * 2.0).powf(1.5);
            (raw_noise * (1.0 - pole_influence)) + (self.pole_heights[1] * pole_influence)
        }
    }

    fn temp_at(&self, pt : Point) -> f32 {
        let d = Self::dist_to_pole(pt[1]) * (1.0-self.height_at(pt));
        (self.temp_equator * (2.0 * d) + self.temp_pole * (2.0 * (0.5-d))
        + (globe_sample(&self.temp_nf, pt) + 1.0)) * 0.5
    }

    // (-1.0, 1.0)
    fn raw_sample(nf : &NoiseField, pt : Point) -> f32 {
        globe_sample(nf, pt)
    }

    pub fn material_at(&self, pt : Point) -> Material {
        let temp = self.temp_at(pt);
        if self.height_at(pt) < self.water_level {
            if temp < -0.2 {Material::Ice}
            else if temp > 1.8 {Material::HotRock}
            else if temp > 1.0 {Material::Rock}
            else {Material::Water}
        } else {
            if temp > 1.4 {Material::HotRock}
            else if temp > 0.9 {Material::Rock}
            else if temp < 0.0 {Material::Snow}
            else {Material::Foliage}
        }
    }


    // (-1.0, 1.0)
    fn slope_at(&self, pt : Point) -> f32 {
        let pt = [pt[0]*AZIMUTH_MULT, pt[1]*AZIMUTH_MULT];
        sigmoid(
            (self.sample(pt) - self.sample([pt[0]+AZIMUTH_SHIFT, pt[1]+AZIMUTH_SHIFT])) * 0.5,
            40.0,
        )
    }

    // (0.0, 1.0)
    fn height_at(&self, pt : Point) -> f32 {
        let x = self.sample(pt) * 0.5 + 0.5;
        // self.height_curve.get(x)// x.powf(1.55)
        x.powf(1.55)
    }

    fn pixel_sample(&self, pt : Point) -> U8Pixel {
        let height = self.height_at(pt);
        // let pole_dist = 1.0 - (0.5 - pt[1]).abs() * 2.0;
        px_finalize(
            {
                let mat = self.material_at(pt);
                // mat.col()
                if mat == Material::Water || mat == Material::Ice {
                    px_shade(mat.col(), 0.3 + (height * 0.7))
                } else {
                    px_shade(mat.col(), 0.2 + ((self.slope_at(pt)*0.5 + 0.5) * 0.8))
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
