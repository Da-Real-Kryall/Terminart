mod colours;

use image::GenericImageView; // to allow calling .pixels()
use colours::PALETTE;
use termion::{terminal_size_pixels, terminal_size};
use clap::{Arg, App};

fn find_closest_colour(clr: &[f64]) -> (usize, [f64; 3]) {
    let mut closest: usize = 0;
    let mut c_dst: f64 = 255.0 * 255.0;
    for cand in 0..PALETTE.len() {
        let n_clr = PALETTE[cand];
        let dst = (((clr[0] as u32).abs_diff(n_clr[0]).pow(2)
            + (clr[1] as u32).abs_diff(n_clr[1]).pow(2)
            + (clr[2] as u32).abs_diff(n_clr[2]).pow(2)) as f64)
            .sqrt();
        if dst < c_dst {
            c_dst = dst;
            closest = cand;
        }
    }
    (
        closest,
        [
            clr[0] - PALETTE[closest][0] as f64,
            clr[1] - PALETTE[closest][1] as f64,
            clr[2] - PALETTE[closest][2] as f64,
        ],
    )
}

fn main() {
    let matches = App::new("Terminartist")
        .about("Converts images to terminal art")
        .arg(Arg::with_name("file")
                 .short('f')
                 .long("file")
                 .takes_value(true)
                 .help("File input"))
        .arg(Arg::with_name("height")
                 .short('h')
                 .long("height")
                 .takes_value(true)
                 .help("Height of output"))
        .get_matches();

    let path = match matches.value_of("file") {
        Some(f) => f,
        None => {
            println!("File must be given with -f.");
            return;
        },
    };

    let image = match image::open(path) {
        Ok(i) => i,
        Err(_) => {
            println!("Error opening image, check the file path given.");
            return;
        },
    };

    let height: u32 = match matches.value_of("height") {
        None => (terminal_size().unwrap().1/2) as u32,
        Some(s) => {
            match s.parse::<u32>() {
                Ok(n) => n,
                Err(_) => {
                    println!("Invalid height given, must be a positive integer.");
                    return;
                }
            }
        }
    };

    
    let terminal_pixel_size = terminal_size_pixels().unwrap();
    let terminal_char_size = terminal_size().unwrap();
    
    let char_width: u32 = u32::from(terminal_pixel_size.0/terminal_char_size.0);
    let char_height: u32 = u32::from(terminal_pixel_size.1/terminal_char_size.1);
    
    let image_pixel_height: u32 = height*char_height;
    let image_pixel_width: u32 = ((image.width() as f64/image.height() as f64)*image_pixel_height as f64) as u32;
    
    let image_char_height: u32 = height.clone()*2;
    let image_char_width: u32 = (image_pixel_width as f64/char_width as f64) as u32;
    
    let mut organised_image_pixels: Vec<Vec<Vec<[f64; 3]>>> = vec![vec![vec![[0.0, 0.0, 0.0]; 0]; image_char_width as usize]; image_char_height as usize];

    for y in 0..image.height() {
        for x in 0..image.width() {
            let _height = organised_image_pixels.len() as f64;
            let _width = organised_image_pixels[0].len() as f64;
            organised_image_pixels[
                (y as f64 / (image.height() as f64 / _height)).min(_height-1.0) as usize
            ][
                (x as f64 / (image.width() as f64 / _width)).min(_width-1.0) as usize
            ].push({
                let px = image.get_pixel(x, y).0;
                [
                    f64::from(px[0]),
                    f64::from(px[1]),
                    f64::from(px[2]),
                ]
            })
        }
    }

    let mut pixel_colours: Vec<Vec<[f64; 3]>> = vec![vec![[0.0; 3]; image_char_width as usize]; image_char_height as usize];

    for y in 0..image_char_height {
        for x in 0..image_char_width {
            let mut sum_pixel = [
                0.0,
                0.0,
                0.0,
            ];
            for add_pixel in &organised_image_pixels[y as usize][x as usize] {
                sum_pixel[0] += add_pixel[0];
                sum_pixel[1] += add_pixel[1];
                sum_pixel[2] += add_pixel[2];
            }
            sum_pixel[0] /= organised_image_pixels[y as usize][x as usize].len() as f64;
            sum_pixel[1] /= organised_image_pixels[y as usize][x as usize].len() as f64;
            sum_pixel[2] /= organised_image_pixels[y as usize][x as usize].len() as f64;

            pixel_colours[y as usize][x as usize] = sum_pixel;
        }
    }

    //now that i have the raw colours of a 4x sized print image, 
    // i now dither and map them to the given colour palette.
    // (i can probably merge this with the above loop)

    //index 256 is transparent


    //twice as tall, but mirrorred y and x
    let mut dithered_pixels: Vec<Vec<usize>> = vec![vec![0; image_char_width as usize]; image_char_height as usize];

    for y in 0..image_char_height {
        for x in 0..image_char_width {
            let old_pixel = pixel_colours[y as usize][x as usize];
            let (new_pixel, quant_error) = find_closest_colour(&old_pixel[0..3]);
        
            if x as usize != pixel_colours[0].len() - 1 && y as usize != pixel_colours.len() - 1 {

                if x != pixel_colours[0].len() as u32 - 1 {
                    pixel_colours[y as usize][1 + x as usize][0] += quant_error[0] * 7.0 / 16.0;
                    pixel_colours[y as usize][1 + x as usize][1] += quant_error[1] * 7.0 / 16.0;
                    pixel_colours[y as usize][1 + x as usize][2] += quant_error[2] * 7.0 / 16.0;
                }
                if x != 0 {
                    pixel_colours[1 + y as usize][x as usize - 1][0] += quant_error[0] * 3.0 / 16.0;
                    pixel_colours[1 + y as usize][x as usize - 1][1] += quant_error[1] * 3.0 / 16.0;
                    pixel_colours[1 + y as usize][x as usize - 1][2] += quant_error[2] * 3.0 / 16.0;
                }
                if y != pixel_colours.len() as u32 - 1 {
                    pixel_colours[1 + y as usize][x as usize][0] += quant_error[0] * 5.0 / 16.0;
                    pixel_colours[1 + y as usize][x as usize][1] += quant_error[1] * 5.0 / 16.0;
                    pixel_colours[1 + y as usize][x as usize][2] += quant_error[2] * 5.0 / 16.0;
                    if x != pixel_colours[0].len() as u32 - 1 {
                        pixel_colours[1 + y as usize][1 + x as usize][0] += quant_error[0] * 1.0 / 16.0;
                        pixel_colours[1 + y as usize][1 + x as usize][1] += quant_error[1] * 1.0 / 16.0;
                        pixel_colours[1 + y as usize][1 + x as usize][2] += quant_error[2] * 1.0 / 16.0;
                    }
                }
            }
            dithered_pixels[y as usize][x as usize] = new_pixel;
        }
    }

    let mut buffer = String::new();
    //evaluate pixels in lots of 2
    for y in 0..dithered_pixels.len()/2 {
        for x in 0..dithered_pixels[0].len() {
            let fg_colour = dithered_pixels[2*y][x]; //▀ top half
            let bg_colour = dithered_pixels[2*y+1][x]; //▄ bottom half
            buffer.push_str(&format!("\x1b[38;5;{};48;5;{}m▀", fg_colour, bg_colour));
        }
        buffer.push_str("\x1b[0m\n");
    }
    print!("{}", buffer);
}
