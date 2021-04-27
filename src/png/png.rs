use std::{cmp::{max, min}, convert::TryInto, fs, mem::transmute, str};
use inflate::inflate_bytes_zlib;

pub use super::{
    chunk::*,
    color::Color,
    scanline::*,
    pixel::*
};

// structs
#[derive(Clone, Debug, Default)]
pub struct Png {
    pub filepath: String,
    pub chunks: Vec<Chunk>,

    // IHDR values
    pub width: u32,
    pub height: u32,
    pub depth: u8,
    pub color_type: u8,
    pub compression_type: u8,
    pub filter: u8,
    pub interface: u8,

    // PLTE (and tRNS)
    pub colors: Vec<Color>,

    // gAMA
    pub gamma: u32,

    // sRGB
    pub rendering_intent: u8,

    // pHYS
    pub ppu_x: u32,
    pub ppu_y: u32,
    pub unit_spec: u8,

    // tRNS
    pub gray_lvl: Option<u16>,
    pub truecolor_alpha: Option<Color>,

    // IDAT
    pub pixels: Vec<Pixel>,
    pub filters: Vec<u8>,

    pub aspect_ratio: f32, 

    raw_bytes: Vec<u8>,
}

// impl
impl Png {
    pub fn read_from_file(filepath: &str) -> Result<Png, String> {
        let mut out: Png = Png::default();

        let bytes: Vec<u8> = fs::read(filepath).expect(&format!("{} could not be read", filepath));
        out.raw_bytes = bytes;

        let img_buffer: &[u8] = out.raw_bytes.as_slice();

        // check header [src: https://en.wikipedia.org/wiki/Portable_Network_Graphics#File_header]
        let expected_header: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

        if img_buffer[0..8] != expected_header {
            return Err(format!("Invalid file header for file: {}", filepath))
        }

        // check for end of image (IEND....) [src: https://en.wikipedia.org/wiki/Portable_Network_Graphics#Critical_chunks]
        let expected_eoi: [u8; 8] = [0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82];

        if img_buffer[img_buffer.len()-8..img_buffer.len()] != expected_eoi {
            return Err(format!("Invalid EOF chunk for file: {}", filepath))
        }
        out.filepath = filepath.to_owned();

        let chunks = Chunk::from_bytes(&out.raw_bytes);
        out.chunks = chunks;

        for chunk in &out.chunks {
            match &chunk.name[..] {
                "IHDR" => {
                    // {width(4), height(4), depth(1), color_type(1), compression(1), filter(1), interface(1)} [src: https://en.wikipedia.org/wiki/Portable_Network_Graphics#Critical_chunks]
                    out.width = unsafe {
                        transmute::<[u8; 4], u32>(chunk.data[0..4].try_into().unwrap()).to_be() 
                    };

                    out.height = unsafe {
                        transmute::<[u8; 4], u32>(chunk.data[4..8].try_into().unwrap()).to_be() 
                    };

                    out.depth = chunk.data[8];
                    out.color_type = chunk.data[9];
                    out.compression_type = chunk.data[10];
                    out.filter = chunk.data[11];
                    out.interface = chunk.data[12];

                    out.aspect_ratio = (out.width as f32) / (out.height as f32);
                },
                "PLTE" => {
                    for c in chunk.data.chunks(3).into_iter() {
                        out.colors.push(Color { r: c[0], g: c[1], b: c[2], a: 255 });
                    }

                    out.colors.dedup();
                },
                "gAMA" => {
                    out.gamma = unsafe {
                        transmute::<[u8; 4], u32>(chunk.data[..].try_into().unwrap()).to_be() 
                    };
                },
                "sRGB" => {
                    out.rendering_intent = chunk.data[0];
                },
                "pHYS" => {
                    out.ppu_x = unsafe {
                        transmute::<[u8; 4], u32>(chunk.data[0..4].try_into().unwrap()).to_be() 
                    };

                    out.ppu_y = unsafe {
                        transmute::<[u8; 4], u32>(chunk.data[4..8].try_into().unwrap()).to_be() 
                    };

                    out.unit_spec = chunk.data[8];
                },
                "tRNS" => {
                    match out.color_type {
                        0 => {
                            out.gray_lvl = unsafe {
                                Some(transmute::<[u8; 2], u16>(chunk.data[..].try_into().unwrap()).to_be())
                            };
                        },
                        2 => {
                            let c: Color;

                            let r = unsafe {
                                transmute::<[u8; 2], u16>(chunk.data[0..2].try_into().unwrap()).to_be() as u8 // this is stupid but it works i think
                            };
                            let g = unsafe {
                                transmute::<[u8; 2], u16>(chunk.data[2..4].try_into().unwrap()).to_be() as u8 // ...
                            };
                            let b = unsafe {
                                transmute::<[u8; 2], u16>(chunk.data[4..6].try_into().unwrap()).to_be() as u8 // ...
                            };

                            c = Color { r, g, b, a: 255 };

                            out.truecolor_alpha = Some(c);
                        },
                        3 => {
                            // len(tRNS <= PLTE) [src: http://libpng.org/pub/png/spec/1.2/PNG-Chunks.html#C.tRNS]
                            for i in 0..min(chunk.data.len(), out.colors.len()) {
                                out.colors[i].a = chunk.data[i];
                            }
                        },
                        _ => {}
                    }
                },
                "IDAT" => {
                    let inflated_bytes = inflate_bytes_zlib(&chunk.data).unwrap();
                    
                    out.pixels = Vec::new();
                    match out.color_type {
                        0 => { 
                            let mut scanlines: Vec<Scanline> = inflated_bytes.chunks((out.width as usize / 8 * out.depth as usize) + 1)
                                                                             .map(|c| Scanline::from_bytes(&c.to_vec(), 1, out.depth))
                                                                             .collect();
                                

                            for i in 0..scanlines.len() {
                                out.filters.push(scanlines[i].filter);
                                let prev_line = if i > 0 { Some(scanlines[i - 1].pixel_bytes.clone()) } else { None };

                                scanlines[i].unfilter(if out.filters[i] == 1 { None } else { prev_line.as_ref() });
                            }

                            for y in 0..(out.height as u32) {
                                for x in 0..(out.width as u32) {
                                    let c: Color = {
                                        let g = scanlines[y as usize].pixel_bytes[x as usize];

                                        Color::grayscale(g, 255)
                                    };

                                    out.pixels.push(Pixel {
                                        color: c,
                                        palette_index: None,
                                        pos: (x, y)
                                    });
                                }
                            }
                        },
                        2 => {
                            let line_size = (out.width as usize * if out.depth == 16 { 6 } else { 3 }) as usize + 1;

                            let mut scanlines: Vec<Scanline> = {
                                if out.interface == 0 {
                                    inflated_bytes.chunks((out.width as usize / 8 * out.depth as usize) + 1)
                                                  .map(|c| Scanline::from_bytes(&c.to_vec(), 1, out.depth))
                                                  .collect()
                                } else {
                                    let adam7 = [
                                        [1, 6, 4, 6, 2, 6, 4, 6],
                                        [7, 7, 7, 7, 7, 7, 7, 7],
                                        [5, 6, 5, 6, 5, 6, 5, 6],
                                        [7, 7, 7, 7, 7, 7, 7, 7],
                                        [3, 6, 4, 6, 3, 6, 4, 6],
                                        [7, 7, 7, 7, 7, 7, 7, 7],
                                        [5, 6, 5, 6, 5, 6, 5, 6],
                                        [7, 7, 7, 7, 7, 7, 7, 7],
                                    ];

                                    let pixel_width = (out.depth as f32 / 8.0).ceil() as usize;
                                    let line_width = 1 + (out.width as usize * pixel_width);

                                    let lines: Vec<Scanline> = Vec::new();

                                    let pass_sizes: Vec<usize> = [
                                        1.0 / 64.0,
                                        1.0 / 64.0,
                                        1.0 / 32.0,
                                        1.0 / 16.0,
                                        1.0 / 8.0,
                                        1.0 / 4.0,
                                        1.0 / 2.0,
                                    ].iter().map(|s| (out.height as f32 * out.width as f32 * s).ceil() as usize * pixel_width).collect();

                                    let pass_widths: Vec<usize> = [
                                        1.0 / 64.0,
                                        1.0 / 64.0,
                                        1.0 / 32.0,
                                        1.0 / 16.0,
                                        1.0 / 8.0,
                                        1.0 / 4.0,
                                        1.0 / 2.0,
                                    ].iter().map(|s| (out.width as f32 * s).ceil() as usize * pixel_width).collect();

                                    let offsets = {
                                        let mut o = [0; 7];

                                        for i in 0..7 {
                                            o[i] = pass_sizes[0..i].iter().sum()
                                        }

                                        o
                                    };

                                    let scanlines = {
                                        let mut s: Vec<Scanline> = Vec::new();

                                        let chunks = vec![
                                            inflated_bytes[0..offsets[1]].to_vec(),
                                            inflated_bytes[offsets[1]..offsets[2]].to_vec(),
                                            inflated_bytes[offsets[2]..offsets[3]].to_vec(),
                                            inflated_bytes[offsets[3]..offsets[4]].to_vec(),
                                            inflated_bytes[offsets[4]..offsets[5]].to_vec(),
                                            inflated_bytes[offsets[5]..offsets[6]].to_vec(),
                                            inflated_bytes[offsets[6]..].to_vec()
                                        ];

                                        for i in 0..6 {
                                            for l in (0..chunks[i].len()).step_by(pass_widths[i]) {
                                                s.push(Scanline::from_bytes(&chunks[i][l..l + pass_widths[i]].to_vec(), pixel_width as u8, out.depth));
                                            }
                                        }

                                        s
                                    };

                                    println!("{:?}", scanlines[0].raw_bytes);

                                    lines
                                }
                            };
                            
                            for i in 0..scanlines.len() {
                                out.filters.push(scanlines[i].filter);
                                let prev_line = if i > 0 { Some(scanlines[i - 1].pixel_bytes.clone()) } else { None };

                                scanlines[i].unfilter(if out.filters[i] == 1 { None } else { prev_line.as_ref() });
                            }

                            for y in 0..(out.height as u32) {
                                for x in (0..(out.width * 3 as u32)).step_by(3) {
                                    let c: Color = {
                                        let r = scanlines[y as usize].pixel_bytes[x as usize];
                                        let g = scanlines[y as usize].pixel_bytes[x as usize + 1];
                                        let b = scanlines[y as usize].pixel_bytes[x as usize + 2];

                                        Color { r, g, b, a: 255u8 }
                                    };

                                    out.pixels.push(Pixel {
                                        color: c,
                                        palette_index: None,
                                        pos: (x / 3, y)
                                    });
                                }
                            }
                        },
                        3 => {
                            let mut scanlines: Vec<Scanline> = inflated_bytes.chunks((out.width as usize * out.depth as usize / 8) as usize + 1)
                                                                         .map(|c| Scanline::from_bytes(&c.to_vec(), 4, out.depth))
                                                                         .collect();

                            for i in 0..scanlines.len() {
                                out.filters.push(scanlines[i].filter);
                                let prev_line = if i > 0 { Some(scanlines[i - 1].pixel_bytes.clone()) } else { None };

                                scanlines[i].unfilter(if out.filters[i] == 1 { None } else { prev_line.as_ref() });
                            }
                            
                            for y in 0..(out.height as u32) {
                                for x in 0..(out.width as u32) {
                                    let pi = scanlines[y as usize].pixel_bytes[x as usize];

                                    out.pixels.push(Pixel::from_palette_index((x, y), pi as usize, &out.colors));
                                }
                            }
                        },
                        4 => {
                            let mut scanlines: Vec<Scanline> = inflated_bytes.chunks((out.width as usize / 4 * out.depth as usize) + 1)
                                                                             .map(|c| Scanline::from_bytes(&c.to_vec(), 2, out.depth))
                                                                             .collect();

                            for i in 0..scanlines.len() {
                                out.filters.push(scanlines[i].filter);
                                let prev_line = if i > 0 { Some(scanlines[i - 1].pixel_bytes.clone()) } else { None };

                                scanlines[i].unfilter(if out.filters[i] == 1 { None } else { prev_line.as_ref() });
                            }

                            for y in 0..(out.height as u32) {
                                for x in (0..(out.width as u32 * 2)).step_by(2) {
                                    let c: Color = {
                                        let g = scanlines[y as usize].pixel_bytes[x as usize];
                                        let a = scanlines[y as usize].pixel_bytes[x as usize + 1];

                                        Color::grayscale(g, a)
                                    };

                                    out.pixels.push(Pixel {
                                        color: c,
                                        palette_index: None,
                                        pos: (x, y)
                                    });
                                }
                            }

                        },
                        6 => {
                            let mut scanlines: Vec<Scanline> = inflated_bytes.chunks((out.width as usize * out.depth as usize / 2) + 1)
                                                                             .map(|c| Scanline::from_bytes(&c.to_vec(), 4, out.depth))
                                                                             .collect();

                            for i in 0..scanlines.len() {
                                out.filters.push(scanlines[i].filter);
                                let prev_line = if i > 0 { Some(scanlines[i - 1].pixel_bytes.clone()) } else { None };

                                scanlines[i].unfilter(if out.filters[i] == 1 { None } else { prev_line.as_ref() });
                            }
                            
                            for y in 0..(out.height as u32) {
                                for x in (0..(out.width * 4 as u32)).step_by(4) {
                                    let c: Color = {
                                        let r = scanlines[y as usize].pixel_bytes[x as usize];
                                        let g = scanlines[y as usize].pixel_bytes[x as usize + 1];
                                        let b = scanlines[y as usize].pixel_bytes[x as usize + 2];
                                        let a = scanlines[y as usize].pixel_bytes[x as usize + 3];

                                        Color { r, g, b, a }
                                    };

                                    out.pixels.push(Pixel {
                                        color: c,
                                        palette_index: None,
                                        pos: (x / 4, y)
                                    });
                                }
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        Ok(out)
    }

    pub fn scale(&mut self, scalar: usize) {
        let mut new_pixels: Vec<Pixel> = Vec::new();
        
        for y in 0..(self.height * scalar as u32) {
            let iy = (y as f32 / scalar as f32).floor() as usize;

            for x in 0..(self.width * scalar as u32) {
                let ix = (x as f32 / scalar as f32).floor() as usize;

                new_pixels.push(self.pixels[ix+iy*self.width as usize]);
            }
        }

        self.pixels = new_pixels;

        self.width *= scalar as u32;
        self.height *= scalar as u32;
    }

    pub fn scale_to_min_width(&mut self, min_width: u32) {
        if max(self.width, min_width) == min_width {
            let s = (min_width as f32 / self.width as f32).floor() as usize;

            self.scale(s);
        }
    }

    pub fn print(&self, full: bool) {
        println!("Reading PNG from: {}", self.filepath);
        println!("Size:             {} bytes", self.raw_bytes.len());
        println!("Chunks: {{");

        println!("  IHDR: {{");
        println!("    width:            {} pixels", self.width);
        println!("    height:           {} pixels", self.height);
        println!("    depth:            {}-bit", self.depth);
        println!("    color_type:       {}", self.color_type);
        println!("    compression_type: {}", self.compression_type);
        println!("    filter:           {}", self.filter);
        println!("    interface:        {}", self.interface);
        println!("  }},");

        println!("  PLTE: {{");
        println!("    colors: {{");
        for i in 0..self.colors.len() {
            println!("      [{:03}]: {}", i, self.colors[i].as_hex());
        }
        if self.colors.len() == 0 {
            println!("      None");
        }
        println!("    }}");
        println!("  }},");

        println!("  gAMA: {{");
        println!("    gamma: {} / 100000", self.gamma);
        println!("  }},");

        println!("  sRGB: {{");
        println!("    rendering_intent: {}", self.rendering_intent);
        println!("  }},");

        println!("  pHYS: {{");
        println!("    ppu_x:     {}", self.ppu_x);
        println!("    ppu_y:     {}", self.ppu_y);
        println!("    unit_spec: {}", self.unit_spec);
        println!("  }},");
        
        println!("  tRNS: {{");
        println!("    gray_lvl:        {}", if self.gray_lvl == None { "None".to_owned() } else { format!("{}", self.gray_lvl.unwrap()) });
        println!("    truecolor_alpha: {}", if self.truecolor_alpha == None { "None".to_owned() } else { format!("{}", self.truecolor_alpha.clone().unwrap().as_hex()) });
        println!("  }},");

        println!("  IDAT: {{");
        println!("    pixels: {{");
        for y in 0..min(self.height, if full { u32::MAX } else { 16 }) {
            print!("      ");
            print!("{}: ", self.filters[y as usize]);

            for x in 0..min(self.width, 8) {
                let p = &self.pixels[(x+y*self.width) as usize];

                print!("[{}] ", p.color.as_hex())
            }
            if self.width > 8 { print!("... (+{})", self.width - 8) }
            println!("");
        }
        if self.height > 8 { println!("      ... (+{})", self.height - 16) }
        println!("    }}");
        println!("  }}");

        println!("}}");
    }
}