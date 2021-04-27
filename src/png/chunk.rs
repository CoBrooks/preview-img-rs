use std::{convert::TryInto, mem::transmute, str};

#[derive(Clone, Debug)]
pub struct Chunk {
    pub length: u32,
    pub name: String,
    pub data: Vec<u8>,
    pub crc: u32
}

impl Chunk {
    pub fn from_bytes(bytes: &Vec<u8>) -> Vec<Chunk> {
        let mut chunks: Vec<Chunk> = Vec::new();
        let chunknames: [&str; 21] = ["IHDR", "PLTE", "IDAT", "IEND", "bKGD", "cHRM", "dSIG", "eXIF", "gAMA", "hIST", "iCCP", "iTXt", "pHYs", "sBIT", "sPLT", "sRGB", "sTER", "tEXT", "tIME", "tRNS", "zTXt"];

        // strip header from bytes
        let mut chunk_bytes: Vec<u8> = Vec::new();
        chunk_bytes.extend_from_slice(&bytes[8..bytes.len()]);

        let mut chunk_offsets: Vec<u32> = Vec::new();

        // find the chunk name offset
        for i in 0..chunk_bytes.len() {
            // IEND has to be the last chunk; break after marking beginning
            if &chunk_bytes[i..i+4] == "IEND".as_bytes() {
                chunk_offsets.push(i as u32);

                break
            }

            let s = str::from_utf8(&chunk_bytes[i..i+4]);
            match s {
                Ok(val) => {
                    if chunknames.contains(&val) {
                        chunk_offsets.push(i as u32);
                    }
                },
                Err(_) => {}
            }
        }

        for &o in &chunk_offsets {
            let i = &chunk_offsets.clone().into_iter().position(|x| x == o).unwrap();

            let length: u32 = unsafe { 
                transmute::<[u8; 4], u32>(chunk_bytes[(o as usize - 4)..(o as usize)].try_into().unwrap()).to_be() 
            };

            let name: String = str::from_utf8(&chunk_bytes[(o as usize)..(o as usize + 4)]).unwrap().to_owned();

            let mut crc_offset: u32 = (chunk_bytes.len() - 4) as u32;
            if i != &(chunk_offsets.len() - 1) {
                crc_offset = chunk_offsets[i+1]
            }

            let crc: u32 = unsafe {
                transmute::<[u8; 4], u32>(chunk_bytes[(crc_offset as usize)..(crc_offset as usize + 4)].try_into().unwrap()).to_be() 
            };


            let mut data: Vec<u8> = chunk_bytes[(o as usize + 4)..(crc_offset as usize)].into_iter().map(|x| *x).collect();
            if i != &(chunk_offsets.len() - 1) {
                data = chunk_bytes[(o as usize + 4)..(crc_offset as usize - 8)].into_iter().map(|x| *x).collect();
            }

            chunks.push(Chunk {
                length: length,
                name: name,
                crc: crc,
                data: data
            });
        }

        chunks
    }
}