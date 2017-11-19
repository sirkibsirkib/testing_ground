use super::procedural::NoiseField;
use super::{Point,Point3D};
use ::rand::{SeedableRng,Rng,Isaac64Rng};
use std::path::Path;
use super::{sigmoid,sig_0_pt5};

extern crate image;

use location::LocationPrimitive;


const AZIMUTH_SHIFT : f32 = 0.006;
const AZIMUTH_MULT : f32 = 1.0 - AZIMUTH_SHIFT;

type FloatPixel = [f32 ; 3];
type U8Pixel = [u8 ; 3];

#[derive(Debug)]
pub struct PointSampleData {
    pub temp : f32,
    pub height : f32,
    pub x_slope : f32,
    pub y_slope : f32,
    pub slope : f32,
}

#[derive(PartialEq,Eq,Copy,Clone,Debug)]
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

#[derive(Debug)]
struct Zone {
    tl : Point,
    br : Point,
    samples : Vec<(PointSampleData,Material)>,
    samples_per_row : u8,
}

impl Zone {
    fn barely_within(&self, pt : Point) -> bool {
        //constructs another bogus zone on the spot which is just a smaller zone inside
        let inner = Zone {
            tl : [self.tl[0]*0.95 + self.br[0]*0.05,
                  self.tl[1]*0.95 + self.br[1]*0.05],
            br : [self.tl[0]*0.05 + self.br[0]*0.95,
                  self.tl[1]*0.05 + self.br[1]*0.95],
            samples : vec![],
            samples_per_row : 99,
        };
        self.within(pt) && !inner.within(pt)
    }

    fn within(&self, other : Point) -> bool {
        (self.tl[0] <= other[0] && other[0] <= self.br[0])
        && (self.tl[1] <= other[1] && other[1] <= self.br[1])
    }

    fn overlaps_with(&self, other : (Point,Point)) -> bool { //other.0 == other.tl
        ! {
            //my left is right of your right
            self.tl[0] > other.1[0]
            //my right is left of your left
            || self.br[0] < other.0[0]
            //my top is below your bottom
            || self.tl[1] > other.1[1]
            //my bottom is above your top
            || self.br[1] < other.0[1]

        }
    }
}

#[derive(Debug)]
pub struct World {
    base_height : NoiseField,
    complex_height : NoiseField,
    temp_nf : NoiseField,
    water_level : f32,
    snow_below_temp : f32,
    grass_within : [f32;2],
    trees_within : [f32;2],
    size : f32,
    zones : Vec<Zone>,
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

        let size = 0.2 + rng.gen::<f32>()*0.25*wp.distance_to_star;
        let radiated_heat = wp.star_energy * (1.0 - wp.distance_to_star);
        let water_level = sig_0_pt5(rng.gen::<f32>() - 0.5 + (size * (1.0 - radiated_heat)), 2.02);
        let base_height_bounds = [
            Self::gen_between(0.05, 0.15, Weighting::Lower(3.0), &mut rng),
            Self::gen_between(0.15, 1.4, Weighting::Lower(2.0), &mut rng),
        ];

        let base_height = NoiseField::generate(&mut rng, base_height_bounds, 4)
        .agglomerate(
            NoiseField::generate(&mut rng, [30.2, 50.4], 2), Some((1.0, 0.013))
        ).agglomerate(
            NoiseField::generate(&mut rng, [2.2, 9.4], 3), Some((1.0, 0.1))
        );

        let complex_height_bounds = [
            Self::gen_between(0.06, 0.4, Weighting::Lower(2.0), &mut rng),
            Self::gen_between(2.5, 5.0, Weighting::Lower(1.2), &mut rng),
        ];

        let complex_height = NoiseField::generate(&mut rng, complex_height_bounds, 4)
        .agglomerate(
            NoiseField::generate(&mut rng, [15.2, 32.4], 3), Some((1.0, 0.12))
        );

        let temp_nf = NoiseField::generate(&mut rng, [30.0, 100.0], 3).agglomerate(
            NoiseField::generate(&mut rng, [0.8, 6.0], 2), Some((1.0, 0.25))
        );

        let mut w = World {
            size : size,
            base_height : base_height,
            complex_height : complex_height,
            temp_nf : temp_nf,
            water_level : water_level,
            snow_below_temp : -sigmoid(-radiated_heat, 4.13),
            grass_within : [0.1,0.2],
            trees_within : [0.1,0.3],
            zones : Vec::new(),
        };
        w.populate_zones(&mut rng);
        println!("got {:?} zones", w.zones.len());
        // println!("world {:#?}", &w);
        w
    }

    fn zone_at(&self, pt : Point) -> Option<&Zone> {
        for z in self.zones.iter() {
            if z.within(pt) {
                return Some(z);
            }
        }
        None
    }

    fn populate_zones<R:Rng>(&mut self, rng : &mut R) {
        let distance_per_x_step = self.size * 0.1;
        let distance_per_y_step = distance_per_x_step * 2.0;

        let mut stop_when_zero : i16 = 500;
        'outer: while stop_when_zero > 0 {
            let tl = [rng.gen::<f32>()*0.9 + 0.05, rng.gen::<f32>()*0.9 + 0.05];
            let (x_steps, y_steps) = (
                rng.gen::<u8>() % 3 + 3,
                rng.gen::<u8>() % 3 + 3,
            );
            let br = [tl[0] + x_steps as f32 * distance_per_x_step, tl[1] + y_steps as f32 * distance_per_y_step];
            if br[0] > 0.95 || br[1] > 0.95 {
                stop_when_zero -= 5;
                continue 'outer;
            }
            for z in self.zones.iter() {
                if z.overlaps_with((tl,br)){
                    stop_when_zero -= 10;
                    continue 'outer;
                }
            }
            let mut samples = vec![];
            let mut count_walkable_materials = 0;
            for y in 0..y_steps {
                for x in 0..x_steps {
                    let (x_offset, y_offset) = (x as f32 * distance_per_x_step, y as f32 * distance_per_y_step);
                    let pt = [tl[0] + x_offset, tl[1] + y_offset];
                    let point_data = self.calc_sample_data_at(pt);
                    let mat = self.material_at(pt, &point_data);
                    match mat {
                        Material::Water | Material::Ice => (),
                        _ => count_walkable_materials += 1,
                    }
                    samples.push((point_data,mat));
                }
            }
            if count_walkable_materials >= 3 {
                // not too much ice|water
                self.zones.push(
                    Zone {
                        tl : tl,
                        br : br,
                        samples : samples,
                        samples_per_row : x_steps,
                    }
                );
                stop_when_zero -= self.zones.len() as i16; //prevent overload
            } else {
                stop_when_zero -= 2;
            }
        }
    }

    fn calc_temp_at(&self, pt : Point, height : f32) -> f32 {
        let x = self.temp_nf.sample_3d(equirectangular(pt)) * 0.5 + 0.5;
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
        if rough_sample > self.water_level {
            let fine_sample = self.complex_height.sample_3d(equirectangular(pt)) * 0.5 + 0.5;
            let fineness = rough_sample - self.water_level;
            fineness * fine_sample + (1.0 - fineness) * rough_sample
        } else {
            rough_sample
        }
    }

    fn pole_distance(y : f32) -> f32 {
        assert!(y <= 1.0 && y >= 0.0);
        if y < 0.5 {y} else {1.0-y}
    }

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
        for (k, v) in self.zones.iter().enumerate() {
            if v.barely_within(pt) {
                return [255,  (k as u8*21 + 200), (k as u8*31)];
            }
        }
        let point_data = self.calc_sample_data_at(pt);
        px_finalize(
            {
                let mat = self.material_at(pt, &point_data);
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
        res
    }
}

const HHH : f32 = 1.0;
pub fn equirectangular(pt : Point) -> Point3D {
    let (x, y) = (pt[0], pt[1]);
    let latitude_radius = (1.0 - (y*2.0 - 1.0).abs()).powf(0.7);
    let l = x * ::std::f32::consts::PI * 2.0;
    [
        l.sin() * latitude_radius * HHH,
        l.cos() * latitude_radius * HHH,
        y * ::std::f32::consts::PI, //PI instead of 2PI to make it 2* wider
    ]
}
