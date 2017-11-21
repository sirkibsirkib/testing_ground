use super::{Material,PointSampleData};
use super::zones::{Zone,ZoneSample};
use ::std::path::Path;
use super::grid::TotalGrid;

const METERS_PER_ZONE_SAMPLE : usize = 50;

pub struct LocationPrimitive {
    seed: u64,
    zone_in_world: Zone,
}

impl LocationPrimitive {
    pub fn new(seed: u64, zone_in_world: Zone) -> LocationPrimitive {
        LocationPrimitive {
            seed: seed,
            zone_in_world: zone_in_world,
        }
    }
}

pub struct Location {
    materials : TotalGrid<Material>,
}

impl Location {
    pub fn generate(loc_prim: LocationPrimitive) -> Location {
        let x_cells =  METERS_PER_ZONE_SAMPLE
            * loc_prim.zone_in_world.get_samples_per_row() as usize;
        let y_cells =  METERS_PER_ZONE_SAMPLE
            * loc_prim.zone_in_world.get_samples_per_col().unwrap() as usize;
        let mut which_mat = |x, y| {
            Material::Water
        };
        Location {
            materials: TotalGrid::new_from_func(x_cells, y_cells, &mut which_mat),
        }
    }
}
