use super::helper::{bytes_as_16bit, bytes_as_1bit, bytes_as_2bit, bytes_as_4bit, bytes_from_16bit_to_8bit};

#[derive(Clone)]
pub struct Scanline {
    pub pixel_length: u8,
    pub pixel_bytes: Vec<u8>,
    pub filter: u8,
    pub depth: u8,
    pub raw_bytes: Vec<u8>
}

impl Scanline {
    pub fn from_bytes(bytes: &Vec<u8>, pixel_length: u8, bit_depth: u8) -> Scanline {
        let pbytes: Vec<u8> = bytes[1..].to_vec(); 
        let mut scanline = Scanline {
            pixel_length,
            pixel_bytes: pbytes,
            filter: bytes[0],
            depth: bit_depth,
            raw_bytes: bytes.clone()
        };

        match bit_depth {
            1  => { scanline.pixel_bytes = bytes_as_1bit(&scanline.pixel_bytes) },
            2  => { scanline.pixel_bytes = bytes_as_2bit(&scanline.pixel_bytes) },
            4  => { scanline.pixel_bytes = bytes_as_4bit(&scanline.pixel_bytes) },
            8  => { /* Bits are 8bit by default */ },
            16 => { scanline.pixel_bytes = bytes_from_16bit_to_8bit(&bytes_as_16bit(&scanline.pixel_bytes)) },
            _  => {}
        }

        scanline
    }

    pub fn unfilter(&mut self, previous_line: Option<&Vec<u8>>) {
        match self.filter {
            0 => {},
            1 => { self.unsub() },
            2 => { self.unup(previous_line.expect("No previous line given for the sub algorithm; aborting")) },
            3 => { self.unaverage(previous_line.expect("No previous line given for the average algorithm; aborting")) },
            4 => { self.unpaeth(previous_line.expect("No previous line given for the paeth algorithm; aborting")) },
            _ => { println!("unrecognized filter type: {}", self.filter) }
        }
    }

    fn unsub(&mut self) {
        let mut unfiltered: Vec<u8> = Vec::new();

        for i in 0..(self.pixel_length as usize) {
            unfiltered.push(self.pixel_bytes[i]);
        }

        for p in (self.pixel_length as usize)..self.pixel_bytes.len() {
            unfiltered.push(((self.pixel_bytes[p] as usize + unfiltered[p - (self.pixel_length as usize)] as usize) % 256) as u8);
        }

        self.pixel_bytes = unfiltered;
    }

    fn unup(&mut self, previous_line: &Vec<u8>) {
        let mut unfiltered: Vec<u8> = Vec::new();

        for i in 0..self.pixel_bytes.len() {
            unfiltered.push(((previous_line[i] as usize + self.pixel_bytes[i] as usize) % 256) as u8)
        }

        self.pixel_bytes = unfiltered;
    }
    
    fn unaverage(&mut self, previous_line: &Vec<u8>) {
        let mut unfiltered: Vec<u8> = Vec::new();

        for i in 0..self.pixel_bytes.len() {
            if (i as isize - self.pixel_length as isize) < 0 {
                unfiltered.push(((self.pixel_bytes[i] as usize + (previous_line[i] as usize / 2)) % 256) as u8);
            } else {
                unfiltered.push((self.pixel_bytes[i] as usize + ((unfiltered[i - self.pixel_length as usize] as usize + previous_line[i] as usize) / 2) % 256) as u8);
            }
        }

        self.pixel_bytes = unfiltered;
    }

    fn unpaeth(&mut self, previous_line: &Vec<u8>) {
        let mut unfiltered: Vec<u8> = Vec::new();

        for i in 0..self.pixel_bytes.len() {
            if (i as isize - self.pixel_length as isize) < 0 {
                let pp = Scanline::paeth_predictor(
                    0,                         // left
                    previous_line[i] as usize, // up
                    0                          // up + left
                );
                unfiltered.push(((self.pixel_bytes[i] as usize + pp) % 256) as u8);            
            } else {
                let pp = Scanline::paeth_predictor(
                    unfiltered   [i-self.pixel_length as usize] as usize, // left
                    previous_line[i] as usize,                            // up
                    previous_line[i-self.pixel_length as usize] as usize  // up + left
                );
                unfiltered.push(((self.pixel_bytes[i] as usize + pp) % 256) as u8);
            }
        }

        self.pixel_bytes = unfiltered;
    }

    // [src: http://libpng.org/pub/png/spec/1.2/PNG-Filters.html]
    fn paeth_predictor(a: usize, b: usize, c: usize) -> usize {
        let a = a as isize; 
        let b = b as isize; 
        let c = c as isize; 

        let p = a + b - c;
        let pa = (p as isize - a).abs() as usize;
        let pb = (p as isize - b).abs() as usize;
        let pc = (p as isize - c).abs() as usize;

        if pa <= pb && pa <= pc { return a as usize; }
        else if pb <= pc { return b as usize; }
        else { return c as usize; }
    }
}