pub fn bytes_as_4bit(bytes: &Vec<u8>) -> Vec<u8> {
    let new_data: Vec<u8> = bytes.iter()
                                    .map(|&b| b.to_u4()[..].to_vec())
                                    .collect::<Vec<Vec<u8>>>()
                                    .iter()
                                    .flatten()
                                    .map(|b| b.to_owned())
                                    .collect::<Vec<u8>>();
    
    new_data
}

pub fn bytes_as_2bit(bytes: &Vec<u8>) -> Vec<u8> {
    let new_data: Vec<u8> = bytes.iter()
                                    .map(|&b| b.to_u2()[..].to_vec())
                                    .collect::<Vec<Vec<u8>>>()
                                    .iter()
                                    .flatten()
                                    .map(|b| b.to_owned())
                                    .collect::<Vec<u8>>();
    
    new_data
}

pub fn bytes_as_1bit(bytes: &Vec<u8>) -> Vec<u8> {
    let new_data: Vec<u8> = bytes.iter()
                                    .map(|&b| b.to_u1()[..].to_vec())
                                    .collect::<Vec<Vec<u8>>>()
                                    .iter()
                                    .flatten()
                                    .map(|b| b.to_owned())
                                    .collect::<Vec<u8>>();

    new_data
}

pub fn bytes_from_16bit_to_8bit(bytes: &Vec<u16>) -> Vec<u8> {
    let new_data: Vec<u8> = bytes.iter()
                                  .map(|b| b.to_u8())
                                  .collect::<Vec<u8>>();
    
    new_data
}

pub fn bytes_as_16bit(bytes: &Vec<u8>) -> Vec<u16> {
    let new_data: Vec<u16> = bytes.chunks(2)
                                  .map(|b| b.to_u16())
                                  .collect::<Vec<u16>>();
    
    new_data
}

pub fn lerp_u8(a: u8, b: u8, f: f32) -> u8 {
    if a == b {
        a
    } else if a < b {
        let diff = b - a;

        a + (diff as f32 * f) as u8
    } else {
        let diff = a - b;

        a - (diff as f32 * f) as u8
    }
}

pub trait UXBigger {
    fn to_u16(&self) -> u16;
}

impl UXBigger for &[u8] {
    fn to_u16(&self) -> u16 {
        (self[0] as u16 * 0x0100) + (self[1] as u16 * 0x0001)
    }
}

impl UXBigger for [u8; 2] {
    fn to_u16(&self) -> u16 {
        (self[0] as u16 * 0x0100) + (self[1] as u16 * 0x0001)
    }
}

pub trait UXSmaller {
    fn to_u8(&self) -> u8;
    fn to_u4(&self) -> [u8; 2];
    fn to_u2(&self) -> [u8; 4]; 
    fn to_u1(&self) -> [u8; 8];
}

impl UXSmaller for u16 {
    fn to_u8(&self) -> u8 {
        (self >> 8) as u8
    }

    fn to_u4(&self) -> [u8; 2] {
        unimplemented!()
    }

    fn to_u2(&self) -> [u8; 4] {
        unimplemented!()
    }

    fn to_u1(&self) -> [u8; 8] {
        unimplemented!()
    }
}

impl UXSmaller for u8 {
    fn to_u8(&self) -> u8 { *self }
    fn to_u4(&self) -> [u8; 2] {
        let mut ret: [u8; 2] = [0; 2];
        let mut index_bit = 0b11110000u8;
        
        for i in 0..2 {
            ret[i] = ((self & index_bit) >> ((1 - i) * 4)) * 0x11;

            index_bit >>= 4;
        }

        ret
    }

    fn to_u2(&self) -> [u8; 4] {
        let mut ret: [u8; 4] = [0; 4];
        let mut index_bit = 0b11000000u8;
        
        for i in 0..4 {
            ret[i] = ((self & index_bit) >> ((3 - i) * 2)) * 0x55;

            index_bit >>= 2;
        }

        ret
    }

    fn to_u1(&self) -> [u8; 8] {
        let mut ret: [u8; 8] = [0; 8];
        let mut index_bit = 0b10000000u8;
        
        for i in 0..8 {
            ret[i] = ((self & index_bit) >> (7 - i)) * 0xFF;

            index_bit >>= 1;
        }

        ret
    }
}