use super::color::Color;

#[derive(Clone, Debug, Copy)]
pub struct Pixel {
    pub pos: (u32, u32),
    pub color: Color,
    pub palette_index: Option<u8>
}

impl Pixel {
    pub fn from_palette_index(pos: (u32, u32), index: usize, palette: &Vec<Color>) -> Pixel {
        Pixel {
            pos,
            color: palette[index],
            palette_index: None
        }
    }
}