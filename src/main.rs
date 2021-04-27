#![allow(unused_variables, non_snake_case, dead_code, unused_imports)]
use std::cmp::max;
use std::io::Cursor;
use std::env;

mod png;
use png::{
    png::Png
};

use pixel_canvas::{Canvas, Color, input::MouseState};

const GRAY_SCALE_1BIT: &str = "test_images/grayscale/basn0g01.png";          // ✓
const GRAY_SCALE_2BIT: &str = "test_images/grayscale/basn0g02.png";          // ✓
const GRAY_SCALE_4BIT: &str = "test_images/grayscale/basn0g04.png";          // ✓
const GRAY_SCALE_8BIT: &str = "test_images/grayscale/basn0g08.png";          // ✓
const GRAY_SCALE_16BIT: &str = "test_images/grayscale/basn0g16.png";         // ✓

const GRAY_SCALE_A_8BIT: &str = "test_images/grayscale_alpha/basn4a08.png";  // ✓
const GRAY_SCALE_A_16BIT: &str = "test_images/grayscale_alpha/basn4a16.png"; // ✓

const PALETTED_1BIT: &str = "test_images/paletted/basn3p01.png";             // ✓
const PALETTED_2BIT: &str = "test_images/paletted/basn3p02.png";             // ✓
const PALETTED_4BIT: &str = "test_images/paletted/basn3p04.png";             // ✓
const PALETTED_8BIT: &str = "test_images/paletted/basn3p08.png";             // ✓

const RGB_8BIT: &str = "test_images/rgb/basn2c08.png";                       // ✓
const RGB_16BIT: &str = "test_images/rgb/basn2c16.png";                      // ✓

const RGB_A_8BIT: &str = "test_images/rgb_alpha/basn6a08.png";               // ✓
const RGB_A_16BIT: &str = "test_images/rgb_alpha/basn6a16.png";              // ✓

const AMOGUS: &str = "test_images/amogus.png";
const BIG_TEST: &str = "test_images/BigTest.png";

const RGB_8BIT_BIG: &str = "test_images/rgb/basn2c08_big.png";
const GRAY_8BIT_BIG: &str = "test_images/grayscale/basn0g08_big.png";

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = args[1].to_owned().replace("\\", "/");
    let mut png = Png::read_from_file(&path).unwrap();

    png.print(false);
    png.scale_to_min_width(500);

    let canvas = Canvas::new(png.width as usize, png.height as usize)
            .title("Viewing: ".to_owned() + &path.split("/").last().unwrap_or("unknown"))
            //.show_ms(true) - Shows ms / frame in titlebar 
            .state(MouseState::new())
            .input(MouseState::handle_input);

    let mut rendered = false;

    canvas.render(move |mouse, image| {
        if !rendered {
            let width = image.width() as usize;
            for (y, row) in image.chunks_mut(width).enumerate() {
                for (x, pixel) in row.iter_mut().enumerate() {
                    let y = (png.height as usize - y) - 1; // vertically flip b/c (0, 0) of the canvas is the bottom-left

                    let p: png::png::Color = png.pixels[x+y*width].color.as_rgb();

                    *pixel = Color {
                        r: p.r,
                        g: p.g,
                        b: p.b,
                    }
                }
            }
        }
        rendered = true;
    });
}

#[allow(unused_imports)]
mod png_tests {
    use super::*;

    #[test]
    fn gray_scale_1_bit() {
        assert!(Png::read_from_file(GRAY_SCALE_1BIT).is_ok());
    }
    #[test]
    fn gray_scale_2_bit() {
        assert!(Png::read_from_file(GRAY_SCALE_2BIT).is_ok());
    }
    #[test]
    fn gray_scale_4_bit() {
        assert!(Png::read_from_file(GRAY_SCALE_4BIT).is_ok());
    }
    #[test]
    fn gray_scale_8_bit() {
        assert!(Png::read_from_file(GRAY_SCALE_8BIT).is_ok());
    }
    #[test]
    fn gray_scale_16_bit() {
        assert!(Png::read_from_file(GRAY_SCALE_16BIT).is_ok());
    }

    #[test]
    fn gray_scale_a_8_bit() {
        assert!(Png::read_from_file(GRAY_SCALE_A_8BIT).is_ok());
    }
    #[test]
    fn gray_scale_a_16_bit() {
        assert!(Png::read_from_file(GRAY_SCALE_A_16BIT).is_ok());
    }
    #[test]
    fn paletted_1_bit() {
        assert!(Png::read_from_file(PALETTED_1BIT).is_ok());
    }
    #[test]
    fn paletted_2_bit() {
        assert!(Png::read_from_file(PALETTED_2BIT).is_ok());
    }
    #[test]
    fn paletted_4_bit() {
        assert!(Png::read_from_file(PALETTED_4BIT).is_ok());
    }
    #[test]
    fn paletted_8_bit() {
        assert!(Png::read_from_file(PALETTED_8BIT).is_ok());
    }
    #[test]
    fn rgb_8_bit() {
        assert!(Png::read_from_file(RGB_8BIT).is_ok());
    }
    #[test]
    fn rgb_16_bit() {
        assert!(Png::read_from_file(RGB_16BIT).is_ok());
    }
    #[test]
    fn rgb_a_8_bit() {
        assert!(Png::read_from_file(RGB_A_8BIT).is_ok());
    }
    #[test]
    fn rgb_a_16_bit() {
        assert!(Png::read_from_file(RGB_A_16BIT).is_ok());
    }
}