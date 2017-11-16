use std::collections::HashMap;


pub type GridID = u64;
pub type BlueprintID = u64;



/*
Server needs a list of ROOMS
server needs a list of WORLDS
server needs to track which rooms each client does and doesn't have loaded
*/

struct Cell {
    x : u32,
    y : u32,
}


struct World {
    seed : u64,
    radius : f32,
    eccentricity : f32,
    rot_period : f32,
    high_temp : f32,
    low_temp : f32,
}

struct Grid<'a> {
    cell_width : f32,
    h_cells : u32,
    v_cells : u32,
    batches : Vec<ObjectBatch<'a>>,
    individuals : HashMap<Cell, Object<'a>>,
}

struct ObjectBatch<'a> {
    blueprint : &'a Blueprint,
    positions : Vec<Cell>,
}

struct Object<'a> {
    position : Cell,
    blueprint : &'a Blueprint,
}


#[derive(Debug,Serialize,Deserialize)]
pub struct Blueprint {
    name : String,
    attributes : Attributes,
}

use self::AttributeType::*;
impl Blueprint {

    // pub fn distance(&self, other : &Blueprint) -> f32 {
    //     self.attributes.distance(&other.attributes)
    // }

    pub fn new(name : String, attributes : Attributes) -> Blueprint {
        Blueprint {
            name : name,
            attributes : attributes,
        }
    }

    #[inline]
    fn get_att(&self, t : AttributeType) -> u32 {
        self.attributes.get(t)
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Attributes {
    a : HashMap<AttributeType, u32>,
}

impl Attributes {
    pub fn calc_area(&self) -> u32 {
        self.get(AttributeType::Length) * self.get(AttributeType::Length)
    }

    pub fn density(&self) -> u32 {
        self.get(AttributeType::Mass) * 4 / self.calc_area()
    }

    fn similarity_coefficient(&self, other : &Attributes) -> f32 {
        let mut similarity = 0.0;
        let mut difference = 0.0;

        for att in self.a.keys() {
            let (a, b) = (self.get(*att) as f32, other.get(*att) as f32);
            difference += (a - b).abs();
            similarity += if a < b {a} else {b};
        }
        for (k, v) in other.a.iter() {
            if ! self.a.contains_key(k) {
                difference += *v as f32;
            }
        }
        similarity / (similarity + difference)
    }

    pub fn new(attributes : Vec<(AttributeType, u32)>) -> Attributes {
        let mut x : HashMap<AttributeType, u32> = HashMap::new();
        for (k,v) in attributes {
            if v != 0 {
                x.insert(k, v);
            }
        }
        Attributes::check_length_width(&mut x);
        Attributes {
            a : x,
        }
    }

    fn check_length_width(x : &mut HashMap<AttributeType, u32>) {
        if let Some(&w) = x.get(&AttributeType::Width) {
            if let Some(&l) = x.get(&AttributeType::Length) {
                if l < w {
                    x.insert(AttributeType::Length, w);
                    x.insert(AttributeType::Width, l);
                }
            } else {
                x.insert(AttributeType::Length, w);
                x.remove(&AttributeType::Width);
            }
        }
    }

    fn same_as_but(&self, but : Vec<(AttributeType,u32)>) -> Attributes {
        let mut ret = Attributes{
            a : HashMap::new(),
        };
        for (k,v) in but {
            if v != 0 {
                ret.a.insert(k,v);
            }
        }
        for (k,v) in self.a.iter() {
            if *v != 0 && ! ret.a.contains_key(k) {
                ret.a.insert(*k,*v);
            }
        }
        Attributes::check_length_width(&mut ret.a);
        ret
    }



    fn get(&self, t : AttributeType) -> u32 {
        match self.a.get(&t) {
            Some(x) => *x,
            None => 0,
        }
    }
}

#[derive(Hash,Eq,PartialEq,Copy,Clone,Debug,Serialize,Deserialize)]
pub enum AttributeType {
    Mass, Flammability, Length, Width, Hardness, Starch, Conductivity,
}

pub enum Process {
    Flatten, Coat,
}

fn concat(v : Vec<&str>) -> String {
    let mut s = String::new();
    for x in v {
        s.push_str(x);
    }
    s
}

impl Process {
    fn apply_to_one(&self, input : &Attributes, bp_name : &str) -> (String, Attributes) {
        use AttributeType::*;
        match self {
            &Process::Flatten => {
                if input.get(Hardness) >= 9 {
                    let side = (input.calc_area() as f32).sqrt() as u32;
                    (
                        concat(vec!["crushed ", bp_name]),
                        Attributes::new(vec![(Mass,input.get(Mass)), (Length,side), (Width,side), (Starch,input.get(Starch))])
                    )
                } else {
                    (
                        concat(vec!["flattened ", bp_name]),
                        input.same_as_but(vec![(Width, input.get(Width)/2), (Hardness, input.get(Hardness)+5)]),
                    )
                }

            }
            &Process::Coat => {
                (
                    concat(vec!["coated ", bp_name]),
                    input.same_as_but(vec![(Conductivity, input.get(Conductivity)/3)]),
                )
            }
        }
    }
}



////////////// SERVER /////////////

#[derive(Debug)]
pub struct BlueprintPool {
    bps : HashMap<BlueprintID, Blueprint>,
    next_id : BlueprintID,
}

impl BlueprintPool {
    pub fn new() -> BlueprintPool {
        BlueprintPool{
            bps : HashMap::new(),
            next_id : 0,
        }
    }

    pub fn find_similar(&self, att : &Attributes) -> Option<BlueprintID> {
        //TODO comparison function and thresh are currently hardcoded
        let sim_thresh = 0.9;

        let mut best : Option<BlueprintID> = None;
        let mut best_sim = 0.0;

        for (bp_id, bp) in self.bps.iter() {
            let sim = att.similarity_coefficient(& bp.attributes);
            if sim > sim_thresh {
                if best.is_none() || best_sim > sim {
                    best = Some(*bp_id);
                    best_sim = sim;
                }
            }
        }
        println!("similarity {:?}", &best_sim);
        best
    }

    pub fn invent(&mut self, bp : Blueprint) -> BlueprintID {
        let bp_id = self.next_id;
        self.next_id += 1;
        self.bps.insert(bp_id, bp);
        bp_id
    }

    pub fn apply_process_to_one(&mut self, bp_id : BlueprintID, p : Process) -> Result<BlueprintID, &'static str> {
        let new_bp = if let Some(bp) = self.bps.get(&bp_id) {
            let (new_name, ideal_atts) = p.apply_to_one(&bp.attributes, &bp.name);
            println!("ATTS {:#?}", &ideal_atts);
            if let Some(similar_bp_id) = self.find_similar(&ideal_atts) {
                return Ok(similar_bp_id)
            } else {
                Blueprint {
                    name : new_name,
                    attributes : ideal_atts,
                }
            }
        } else {
            return Err("No such BPID!")
        };
        Ok(self.invent(new_bp))
    }
}
