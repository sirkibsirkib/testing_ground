
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
