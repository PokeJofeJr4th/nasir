use std::io::Cursor;

use image::io::Reader as ImageReader;
use image::{Pixel, RgbaImage};

use crate::types::TerminalLine;
use crate::utils::rgb_to_256;

/// pure function to get an image from the byte stream
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
    let (img_width, img_height) = img.dimensions();
    let (term_width, term_height) =
        get_img_viewport((img_width * 2, img_height), (term_size.0 - 1, term_size.1));
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
                ..=(((col + 1) * img_width / term_width).min(img_width - 1));
            if verbose {
                print!("image columns: {col_iter:?}\r\n");
            }
            let mut pixels_in_chunk = Vec::new();
            for y in row_iter.clone() {
                for x in col_iter.clone() {
                    pixels_in_chunk.push(img.get_pixel(x, y).channels());
                }
            }
            let pix_len = pixels_in_chunk.len().max(1);
            let (mut r, mut g, mut b) = (0, 0, 0);
            for px in pixels_in_chunk {
                r += px[0] as usize * px[3] as usize / 256;
                g += px[1] as usize * px[3] as usize / 256;
                b += px[2] as usize * px[3] as usize / 256;
            }
            let (r, g, b) = (r / pix_len, g / pix_len, b / pix_len);
            current_line.push_str(&format!(
                "\x1b[38;5;{};38;2;{r};{g};{b}m█\x1b[0m",
                // "\x1b[38;5;{}m█\x1b[0m",
                rgb_to_256((r, g, b))
            ));
        }
        termlines.push(TerminalLine::from(current_line));
    }
    termlines
}

/// given the size of an image and the terminal, figure out how big to make the image
const fn get_img_viewport((img_w, img_h): (u32, u32), (term_w, term_h): (u32, u32)) -> (u32, u32) {
    // start with the image size
    // if the image is too tall, squash it
    let (img_w_2, img_h_2) = if img_h > term_h {
        (img_w * term_h / img_h, term_h)
    } else {
        (img_w, img_h)
    };
    // if the image is too long, squish it
    if img_w_2 > term_w {
        (term_w, img_h_2 * term_w / img_w_2)
    } else {
        (img_w_2, img_h_2)
    }
}

#[cfg(test)]
mod tests {
    use crate::img::get_img_viewport;

    #[test]
    fn img_viewport() {
        assert_eq!(get_img_viewport((2, 2), (4, 4)), (2, 2));
        assert_eq!(get_img_viewport((4, 4), (2, 2)), (2, 2));
        assert_eq!(get_img_viewport((2, 4), (4, 4)), (2, 4));
        assert_eq!(get_img_viewport((10, 10), (8, 4)), (4, 4));
        assert_eq!(get_img_viewport((20, 10), (8, 4)), (8, 4));
    }
}
