use std::io::Cursor;

use image::io::Reader as ImageReader;
use image::{Pixel, RgbaImage};

use crate::types::TerminalLine;

/// get an image from the byte stream
pub fn get_image(bytes: &[u8]) -> Result<RgbaImage, String> {
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| String::from("Format Error"))?
        .decode()
        .map_err(|_| String::from("Decoding Error"))?
        .to_rgba8();
    if img.dimensions() == (0, 0) {
        Err(String::from("Empty Image"))
    } else {
        Ok(img)
    }
}

/// approximate the image as a list of terminal lines, given a specific terminal size
pub fn approximate_image(
    img: &RgbaImage,
    term_size: (u32, u32),
    verbose: bool,
) -> Vec<TerminalLine> {
    let term_width = term_size.0 - 1;
    let term_height = term_size.1;
    let (img_width, img_height) = img.dimensions();
    let mut termlines = Vec::new();
    for row in 0..(term_height - 1) {
        let row_iter = (row * img_height / term_height)
            ..=(((row + 1) * img_height / term_height).min(img_height - 1));
        if verbose {
            print!("image rows: {row_iter:?}\r\n");
        }
        let mut current_line = String::new();
        for col in 0..term_width {
            let col_iter = (col * img_width / term_width)
                ..=(((col + 1) * img_width / term_width).min(img_height - 1));
            let mut pixels_in_chunk = Vec::new();
            for y in row_iter.clone() {
                for x in col_iter.clone() {
                    pixels_in_chunk.push(img.get_pixel(x, y).channels());
                }
            }
            let pix_len = pixels_in_chunk.len();
            let (mut r, mut g, mut b) = (0, 0, 0);
            for px in pixels_in_chunk {
                r += px[0] as usize * px[3] as usize / 256;
                g += px[1] as usize * px[3] as usize / 256;
                b += px[2] as usize * px[3] as usize / 256;
            }
            let (r, g, b) = (r / pix_len, g / pix_len, b / pix_len);
            current_line.push_str(&format!("\x1b[38;2;{r};{g};{b}m█\x1b[0m"));
        }
        termlines.push(TerminalLine::from(current_line));
    }
    termlines
}
