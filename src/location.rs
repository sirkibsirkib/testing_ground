use super::world::{Material,PointSampleData};


pub struct LocationPrimitive {
    guideline_samples : Vec<PointSampleData>,
    samples_per_row : usize,
    meters_per_step : f32,
}
