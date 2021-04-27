use std::{cmp::{max, min}, fmt, ops};
use super::helper::lerp_u8;

#[derive(Clone, PartialEq, Copy, Default)]
pub struct Color { 
    pub r: u8, 
    pub g: u8, 
    pub b: u8, 
    pub a: u8 
}

impl Color {
    pub fn as_hex(&self) -> String {
        format!("#{:02x?}{:02x?}{:02x?} {:02x?}", self.r, self.g, self.b, self.a)
    }

    pub fn from_16bit(r: u16, g: u16, b: u16, a: u16) -> Color {
        Color {
            r: (r >> 8) as u8,
            g: (g >> 8) as u8,
            b: (b >> 8) as u8,
            a: (a >> 8) as u8,
        }
    }

    pub fn grayscale(x: u8, a: u8) -> Color {
        Color {
            r: x,
            g: x,
            b: x,
            a
        }
    }

    pub fn as_rgb(&self) -> Color {
        if self.a == 255 {
            *self
        } else {
            let mut c = self.clone().to_owned();
            let f = c.a as f32 / 255.0;

            c.lerp(&Color::grayscale(255, 255), 1.0 - f);

            c.a = 255;

            c
        }
    }

    pub fn lerp(&mut self, other: &Color, amount: f32) {
        let c: Color = {
            let r = lerp_u8(self.r, other.r, amount);
            let g = lerp_u8(self.g, other.g, amount);
            let b = lerp_u8(self.b, other.b, amount);
            let a = lerp_u8(self.a, other.a, amount);

            Color { r, g, b, a}
        };

        *self = c;
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Self::Output {
        Color { 
            r: ((self.r + rhs.r as u8) as usize % 256) as u8, 
            g: ((self.g + rhs.g as u8) as usize % 256) as u8, 
            b: ((self.b + rhs.b as u8) as usize % 256) as u8, 
            a: ((self.a + rhs.a as u8) as usize % 256) as u8 
        }
    }
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Color")
         .field("hex", &format!("#{:02x?}{:02x?}{:02x?}", self.r, self.g, self.b))
         .field("alpha", &format!("{:02x?}", self.a))
         .finish()
    }
}