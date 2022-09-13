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
                pixel_colours[y as usize][1 + x as usize][0] += quant_error[0] * 7.0 / 16.0;
                pixel_colours[y as usize][1 + x as usize][1] += quant_error[1] * 7.0 / 16.0;
                pixel_colours[y as usize][1 + x as usize][2] += quant_error[2] * 7.0 / 16.0;
                
                if x != 0 {
                    pixel_colours[1 + y as usize][x as usize - 1][0] += quant_error[0] * 3.0 / 16.0;
                    pixel_colours[1 + y as usize][x as usize - 1][1] += quant_error[1] * 3.0 / 16.0;
                    pixel_colours[1 + y as usize][x as usize - 1][2] += quant_error[2] * 3.0 / 16.0;
                }
            
                pixel_colours[1 + y as usize][x as usize][0] += quant_error[0] * 5.0 / 16.0;
                pixel_colours[1 + y as usize][x as usize][1] += quant_error[1] * 5.0 / 16.0;
                pixel_colours[1 + y as usize][x as usize][2] += quant_error[2] * 5.0 / 16.0;
            
                pixel_colours[1 + y as usize][1 + x as usize][0] += quant_error[0] * 1.0 / 16.0;
                pixel_colours[1 + y as usize][1 + x as usize][1] += quant_error[1] * 1.0 / 16.0;
                pixel_colours[1 + y as usize][1 + x as usize][2] += quant_error[2] * 1.0 / 16.0;
            }
            dithered_pixels[y as usize][x as usize] = new_pixel;
        }
    }

    //include alpha in quadruplets
    //quadruplets of the dithered colours
    //let mut quadruplet_pixels: Vec<Vec<[usize; 4]>> = vec![vec![[0; 4]; image_char_width as usize / 2]; image_char_height as usize / 2];
    //for y in 0..image_char_height as usize/2 {
    //    for x in 0..image_char_width as usize/2 {
    //        quadruplet_pixels[y][x] = [
    //            dithered_pixels[y][x],
    //            dithered_pixels[y][x+1],
    //            dithered_pixels[y+1][x],
    //            dithered_pixels[y+1][x+1],
    //        ];
    //    }
    //}

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

    //for y in 0..quadruplet_pixels.len() { //here i'll evaluate each quadruplet
    //    for x in 0..quadruplet_pixels[0].len() {
    //        let quadruplet = quadruplet_pixels[y][x];
    //        //and then write the resulting character to the buffer
    //        let mut colours: Vec<usize> = Vec::new();
    //        if !colours.contains(&quadruplet[0]) {
    //            colours.push(quadruplet_pixels[y][x][0]);
    //        }
    //        if !colours.contains(&quadruplet[1]) {
    //            colours.push(quadruplet[1]);
    //        }
    //        if !colours.contains(&quadruplet[2]) {
    //            colours.push(quadruplet[2]);
    //        }
    //        if !colours.contains(&quadruplet[3]) {
    //            colours.push(quadruplet[3]);
    //        }
    //        while colours.len() > 2 {
    //            //let index = (y+x)%4;
    //            for index in 0..4 {
    //                let i = (index+y+x)%4;
    //                let mut num = 0;
    //                //check if the colour that occurs at this index in quadruplet_pixels[y][x] only appears once.
    //                if quadruplet[i] == quadruplet[0] {
    //                    num += 1;
    //                }
    //                if quadruplet[i] == quadruplet[1] {
    //                    num += 1;
    //                }
    //                if quadruplet[i] == quadruplet[2] {
    //                    num += 1;
    //                }
//
//
//
    //            }
    //        }
    //        //indexes of quadruplet colours within colours
    //        //let arrangement: [usize; 4] = [
    //        //    colours.iter().position(|&x| x == quadruplet_pixels[y][x][0]).unwrap(),
    //        //    colours.iter().position(|&x| x == quadruplet_pixels[y][x][1]).unwrap(),
    //        //    colours.iter().position(|&x| x == quadruplet_pixels[y][x][2]).unwrap(),
    //        //    colours.iter().position(|&x| x == quadruplet_pixels[y][x][3]).unwrap(),
    //        //];
    //        //match colours.len() {
    //        //    1 => {
    //        //        buffer.push_str(&format!("\x1b[38;5{}m█", colours[0]));
    //        //    },
    //        //    2 => {
    //        //        match arrangement
    //        //    },
    //        //    3 => {
    //        //    },
    //        //    4 => {
    //        //    },
    //        //    _ => {}
    //        //}
    //    }
    //    buffer.push_str("\x1b[0m\n");
    //}
    //print!("{}", buffer);
    //for y in 0..print_pixels.len() {
    //    for x in 0..print_pixels[0].len() { 
    //        let old_pixel = print_pixels[y][x];
    //        let (new_pixel, quant_error) = find_closest_colour(&old_pixel[0..3]);
    //    
    //        if x != print_pixels[0].len() - 1 && y != print_pixels.len() - 1 {
    //            print_pixels[y as usize][1 + x as usize][0] += quant_error[0] * 7.0 / 16.0;
    //            print_pixels[y as usize][1 + x as usize][1] += quant_error[1] * 7.0 / 16.0;
    //            print_pixels[y as usize][1 + x as usize][2] += quant_error[2] * 7.0 / 16.0;
    //            
    //            if x != 0 {
    //                print_pixels[1 + y as usize][x as usize - 1][0] += quant_error[0] * 3.0 / 16.0;
    //                print_pixels[1 + y as usize][x as usize - 1][1] += quant_error[1] * 3.0 / 16.0;
    //                print_pixels[1 + y as usize][x as usize - 1][2] += quant_error[2] * 3.0 / 16.0;
    //            }
    //        
    //            print_pixels[1 + y as usize][x as usize][0] += quant_error[0] * 5.0 / 16.0;
    //            print_pixels[1 + y as usize][x as usize][1] += quant_error[1] * 5.0 / 16.0;
    //            print_pixels[1 + y as usize][x as usize][2] += quant_error[2] * 5.0 / 16.0;
    //        
    //            print_pixels[1 + y as usize][1 + x as usize][0] += quant_error[0] * 1.0 / 16.0;
    //            print_pixels[1 + y as usize][1 + x as usize][1] += quant_error[1] * 1.0 / 16.0;
    //            print_pixels[1 + y as usize][1 + x as usize][2] += quant_error[2] * 1.0 / 16.0;
    //        }
    //        buffer.push_str(&format!("\x1b[48;5;{}m ", new_pixel));
    //    }
    //    buffer.push_str("\x1b[0m\n");
    //}
    //print!("{}", buffer);
}
