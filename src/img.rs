use std::io::Cursor;

use image::io::Reader as ImageReader;
use image::{Pixel, RgbImage};

use crate::types::TerminalLine;

/// get an image from the url
pub fn get_image(bytes: &[u8]) -> Result<RgbImage, String> {
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| String::from("Format Error"))?
        .decode()
        .map_err(|_| String::from("Decoding Error"))?
        .to_rgb8();
    if img.dimensions() == (0, 0) {
        Err(String::from("Empty Image"))
    } else {
        Ok(img)
    }
}

/// approximate the image as a list of terminal lines, given a specific terminal size
pub fn approximate_image(
    img: &RgbImage,
    term_size: (u32, u32),
    verbose: bool,
) -> Vec<TerminalLine> {
    let term_width = term_size.0 / 2 - 1;
    let term_height = term_size.1;
    let (img_width, img_height) = img.dimensions();
    let mut termlines = Vec::new();
    for row in 0..(term_height - 1) {
        let row_iter = (row * img_height / term_height)..((row + 1) * img_height / term_height);
        if verbose {
            println!("image rows: {row_iter:?}");
        }
        let mut current_line = String::new();
        for col in 0..term_width {
            let col_iter = (col * img_width / term_width)..((col + 1) * img_width / term_width);
            let mut pixels_in_chunk = Vec::new();
            for y in row_iter.clone() {
                for x in col_iter.clone() {
                    pixels_in_chunk.push(img.get_pixel(x, y).channels());
                }
            }
            let pix_len = pixels_in_chunk.len();
            let (mut r, mut g, mut b) = (0, 0, 0);
            for px in pixels_in_chunk {
                r += px[0] as usize;
                g += px[1] as usize;
                b += px[2] as usize;
            }
            let (r, g, b) = (r / pix_len, g / pix_len, b / pix_len);
            current_line.push_str(&format!("\x1b[38;2;{r};{g};{b}m██\x1b[0m"));
        }
        termlines.push(TerminalLine::from(current_line));
    }
    termlines
}
