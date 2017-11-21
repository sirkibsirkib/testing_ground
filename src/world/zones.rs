
use super::{Point,Material,PointSampleData,World,pt_wider_dist};
use::rand::{Rng};
use std::collections::{HashMap};

#[derive(Debug)]
pub struct ZoneSample {
    pt: Point,
    data: PointSampleData,
    mat: Material,
}

pub struct Zone {
    tl: Point,
    br: Point,
    samples: HashMap<Coord, ZoneSample>,
    samples_per_row: u8,
}


fn traversible_link(zones: &Vec<Zone>, a: Point, b: Point, by_land: bool, w: &World) -> bool {
    let pt_checks : u8 = (pt_wider_dist(a, b)*30.0) as u8;
    for c in 0..pt_checks {
        let a_ratio = (c+1) as f32 / (pt_checks + 1) as f32;
        let b_ratio = 1.0 - a_ratio;
        let checked_pt = [
            a[0]*a_ratio + b[0]*b_ratio,
            a[1]*a_ratio + b[1]*b_ratio,
        ];

        let mat = w.material_at(checked_pt, &w.calc_sample_data_at(checked_pt));
        if by_land != mat.is_land() {
            // water in way / land in way
            return false;
        }
        for z in zones.iter() {
            if z.within(checked_pt){
                //The link crosses some other zone!
                return false;
            }
        }
    }
    true
}

impl Zone {
    pub fn get_samples_per_row(&self) -> u8 {
        self.samples_per_row
    }

    pub fn get_samples_per_col(&self) -> Option<u8> {
        let x = self.samples.keys().len() / (self.samples_per_row as usize);
        if x*(self.samples_per_row as usize) == self.samples.len() && x <= 255 {
            Some(x as u8)
        } else {
            None
        }
    }

    pub fn coord_is_left(&self, c : Coord) -> bool {c[0] == 0}
    pub fn coord_is_right(&self, c : Coord) -> bool {c[0] == self.samples_per_row-1}
    pub fn coord_is_top(&self, c : Coord) -> bool {c[1] == 0}
    pub fn coord_is_bottom(&self, c : Coord) -> bool {
        c[1] == self.get_samples_per_col().expect("Not divisible!")-1
    }

    pub fn coord_is_edge(&self, c : Coord) -> bool {
        self.coord_is_left(c)
        || self.coord_is_right(c)
        || self.coord_is_top(c)
        || self.coord_is_bottom(c)
    }

    pub fn coord_is_corner(&self, c : Coord) -> bool {
        (self.coord_is_left(c) || self.coord_is_right(c))
        && (self.coord_is_top(c) || self.coord_is_bottom(c))
    }

    fn boundary_sample_iter<'a>(&'a self) -> Box<Iterator<Item=(Coord,&ZoneSample)> + 'a> {
        Box::new(
            self.samples.iter()
            .filter(move |k_v| {
                self.coord_is_edge(*k_v.0)
            }).map(|k_v| (*k_v.0, k_v.1))
        )
    }

    fn shortest_sample_link(
        &self,
        self_taken: &[Coord],
        other: &Zone,
        other_taken: &[Coord],
        zones: &Vec<Zone>,
        w: &World,
    ) -> Option<WorldLink> {
        let mut shortest: Option<WorldLink> = None;
        for (m_coord, my_sample) in self.boundary_sample_iter() {
            if inside_slice(&m_coord, self_taken) || self.coord_is_corner(m_coord) {continue}
            for (t_coord, their_sample) in other.boundary_sample_iter() {
                if inside_slice(&t_coord, other_taken) || other.coord_is_corner(t_coord) {continue}
                let dist = pt_wider_dist(my_sample.pt, their_sample.pt);

                //only allow links where the material allows for it
                if my_sample.mat.is_land() != their_sample.mat.is_land()
                || dist > WorldLink::MAX_LEN
                || ! traversible_link(zones,  my_sample.pt, their_sample.pt, my_sample.mat.is_land(), w) {
                    continue;
                }

                if let Some(ref previous) = shortest {
                    if previous.length() < dist {continue}
                }
                shortest = Some(WorldLink {
                    zone_a_coord: m_coord,
                    zone_b_coord: t_coord,
                    world_a_pt: my_sample.pt,
                    world_b_pt: their_sample.pt,
                    mat_a: my_sample.mat,
                    mat_b: their_sample.mat,
                    land_link: their_sample.mat.is_land(),
                });
            }
        }
        shortest
    }

    pub fn new(tl: Point, br: Point, samples: HashMap<Coord, ZoneSample>, samples_per_row: u8) -> Zone {
        Zone {
            tl: tl,
            br: br,
            samples: samples,
            samples_per_row: samples_per_row,
        }
    }

    pub fn barely_within(&self, pt: Point) -> bool {
        //constructs another bogus zone on the spot which is just a smaller zone inside
        let inner = Zone {
            tl: [self.tl[0]*0.95 + self.br[0]*0.05,
                  self.tl[1]*0.95 + self.br[1]*0.05],
            br: [self.tl[0]*0.05 + self.br[0]*0.95,
                  self.tl[1]*0.05 + self.br[1]*0.95],
            samples: HashMap::new(),
            samples_per_row: 99,
        };
        self.within(pt) && !inner.within(pt)
    }

    pub fn within(&self, other: Point) -> bool {
        (self.tl[0] <= other[0] && other[0] <= self.br[0])
        && (self.tl[1] <= other[1] && other[1] <= self.br[1])
    }

    pub fn overlaps_with(&self, other: (Point,Point)) -> bool { //other.0 == other.tl
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
// use::std::io::write;
impl::std::fmt::Debug for Zone {
    fn fmt(&self, f: &mut::std::fmt::Formatter) -> Result<(),::std::fmt::Error> {
        write!(f, "tl:{:?}, br:{:?}, ({},{})",
            self.tl, self.br, self.samples_per_row, self.get_samples_per_col().unwrap()
        )
    }
}

pub fn generate_zones_for<R:Rng>(w: &World, rng: &mut R) -> Vec<Zone> {
    let distance_per_x_step = w.get_size() * 0.12;
    let distance_per_y_step = distance_per_x_step * 2.0;

    let mut stop_when_zero: i16 = 500;
    let mut zones: Vec<Zone> = vec![];
    'outer: while stop_when_zero > 0 {
        let tl = [rng.gen::<f32>()*0.9 + 0.05, rng.gen::<f32>()*0.9 + 0.05];
        let (x_steps, y_steps) = (
            rng.gen::<u8>() % 3 + 3,
            rng.gen::<u8>() % 3 + 3,
        );
        // (steps-1) to make sure the last sample points are ON the right/bottom edges
        let br = [tl[0] + (x_steps-1) as f32 * distance_per_x_step, tl[1] + (y_steps-1) as f32 * distance_per_y_step];
        if br[0] > 0.95 || br[1] > 0.95 {
            stop_when_zero -= 1;
            continue 'outer;
        }
        for z in zones.iter() {
            if z.overlaps_with((tl,br)){
                //OVERLAP
                stop_when_zero -= 7;
                continue 'outer;
            }
        }
        let mut samples: HashMap<Coord,ZoneSample> = HashMap::new();
        let mut count_walkable_materials = 0;
        for y in 0..y_steps {
            for x in 0..x_steps {
                let coord = [x,y];
                let (x_offset, y_offset) = (x as f32 * distance_per_x_step, y as f32 * distance_per_y_step);
                let pt = [tl[0] + x_offset, tl[1] + y_offset];
                let point_data = w.calc_sample_data_at(pt);
                let mat = w.material_at(pt, &point_data);
                match mat {
                    Material::Water | Material::Ice => (),
                    _ => count_walkable_materials += 1,
                }
                samples.insert(coord, ZoneSample{data:point_data, pt:pt, mat:mat});
            }
        }
        if count_walkable_materials >= 5 {
            // not too much ice|water
            zones.push(
                Zone::new(tl, br, samples, x_steps)
            );
            stop_when_zero -= zones.len() as i16 + 1; //prevent overload
        } else {
            stop_when_zero -= 2;
        }
        if zones.len() >= 3 {stop_when_zero -= 2}
        if zones.len() >= 5 {stop_when_zero -= 4}
        if zones.len() >= 7 {stop_when_zero -= 10}
    }
    zones
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub struct WorldLink {
    zone_a_coord: Coord,
    zone_b_coord: Coord,
    world_a_pt: Point,
    world_b_pt: Point,
    mat_a: Material,
    mat_b: Material,
    land_link: bool,
}


fn inside_slice<T: PartialEq>(x: &T, slice: &[T]) -> bool {
    for q in slice.iter() {
        if x==q {return true}
    }
    false
}

type Coord = [u8;2];

impl WorldLink {
    const MAX_LEN : f32 = 0.17;
    pub fn length(&self) -> f32 {
        pt_wider_dist(self.world_a_pt, self.world_b_pt)
    }
    pub fn get_world_a_pt(&self) -> Point {self.world_a_pt}
    pub fn get_world_b_pt(&self) -> Point {self.world_b_pt}
    pub fn get_mat_a(&self) -> Material{self.mat_a}
    pub fn get_mat_b(&self) -> Material{self.mat_b}
}

pub fn generate_links_for<R:Rng>(zones: &Vec<Zone>, rng: &mut R, w : &World) -> Vec<WorldLink> {
    let mut z : Vec<_> = zones.iter().collect();
    rng.shuffle(&mut z);
    let mut links: Vec<WorldLink> = vec![];
    let mut taken_samples: Vec<Vec<Coord>> = zones.iter().map(|_| vec![]).collect();
    for (i, zone_i) in zones.iter().enumerate() {
        'pair_loop: for (j, zone_j) in zones.iter().enumerate().skip(i+1) {
            if let Some(shortest) = zone_i.shortest_sample_link(
                &taken_samples[i], zone_j, &taken_samples[j], zones, w
            ) {
                if rng.gen_weighted_bool(6) {
                    // ignore connections randomly
                    continue
                }
                if shortest.length() <= WorldLink::MAX_LEN {
                    taken_samples[i].push(shortest.zone_a_coord);
                    taken_samples[j].push(shortest.zone_b_coord);
                    links.push(shortest);
                }
            }
        }
    }
    links
}
