pub struct TotalGrid<T> {
    elements: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> TotalGrid<T> {
    pub fn new_from_func(width: usize, height: usize, f: &mut FnMut(usize, usize) -> T) -> TotalGrid<T>{
        let mut v = Vec::with_capacity(width*height);
        for x in 0..width {
            for y in 0..height {
                v.push(f(x, y));
            }
        }
        TotalGrid {
            elements : v,
            width: width,
            height: height,
        }
    }

    // pub fn new<C:Copy>(width: usize, height: usize, default: C) -> TotalGrid<C> {
    //     TotalGrid<C>::new_from_func<C>(width,height,&mut (|_,_| default))
    // }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        &mut self.elements[x*y]
    }

    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.elements[x*y]
    }

    pub fn maybe_get(&self, x: usize, y: usize) -> Option<&T> {
        self.elements.get(x*y)
    }

    pub fn maybe_get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.elements.get_mut(x*y)
    }

    pub fn put(&mut self, x: usize, y: usize, element: T) {
        self.elements[x*y] = element;
    }
}

pub struct TotalGridBuilder<T> {
    elements: Vec<T>,
}

impl<T> TotalGridBuilder<T> {
    pub fn new() -> TotalGridBuilder<T> {
        TotalGridBuilder {
            elements: vec![],
        }
    }

    pub fn append(&mut self, element: T) {
        self.elements.push(element);
    }

    pub fn replace(&mut self, element: T) {
        self.elements.push(element);
    }

    pub fn finalize(self, width: usize, height: usize) -> TotalGrid<T> {
        assert_eq!(width*height, self.elements.len());
        TotalGrid {
            elements: self.elements,
            width: width,
            height: height,
        }
    }
}
