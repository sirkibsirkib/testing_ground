use std::fmt;

pub struct Asciireen {
    arr : Vec<u8>,
    width : u8,
    height : u8,
}

impl Asciireen {
    pub fn new(width : u8, height : u8) -> Asciireen {
        let cap = width as usize * height as usize;
        let mut v = Vec::with_capacity(cap);
        for _ in 0..cap {
            v.push(0);
        }
        Asciireen {
            arr : v,
            width : width,
            height : height,
        }
    }

    #[inline]
    fn index(&self, x : u8, y : u8) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);
        x as usize + (self.width as usize * y as usize)
    }

    pub fn set(&mut self, x : u8, y : u8, val : u8) {
        let index = self.index(x, y);
        self.arr[index] = val;
    }

    pub fn get(&self, x : u8, y : u8) -> u8 {
        self.arr[self.index(x, y)]
    }

    pub fn print_func<F>(&self, func : F)
    where F : Fn(u8) -> char {
        for j in 0..self.height {
            for i in 0..self.width {
                print!("{}", func(self.get(i, j)));
            }
            print!(" ");
            for i in 0..10 {
                print!("{}", func(self.get(i, j)));
            }
            println!();
        }
    }
}

impl fmt::Display for Asciireen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for j in 0..self.height {
            for i in 0..self.width {
                let _ = write!(f, " {0: <3}", self.get(i, j));
            }
            let _ = write!(f, "\n\n");
        }
        Ok(())
    }
}
