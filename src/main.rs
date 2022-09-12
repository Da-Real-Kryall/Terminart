mod colours;
use image::GenericImageView; // to allow calling .pixels()
use colours::PALETTE;
use termion::{terminal_size_pixels, terminal_size};

//height of resulting image in characters
const HEIGHT: u32 = 32;

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


//what do i want: the size ratio of ONE CHARACTER's width to height (width/height)
//character width = screen pixel width / width in characters


fn main() {

    let image = image::open("/Users/codyryall/Desktop/code.png").expect("File not found!");
    
    let terminal_pixel_size = terminal_size_pixels().unwrap();
    let terminal_char_size = terminal_size().unwrap();
    
    let char_width: u32 = u32::from(terminal_pixel_size.0/terminal_char_size.0);
    let char_height: u32 = u32::from(terminal_pixel_size.1/terminal_char_size.1);
    
    let image_pixel_height: u32 = HEIGHT*char_height;
    let image_pixel_width: u32 = (image.width()/image.height())*image_pixel_height;
    
    let image_char_height: u32 = HEIGHT.clone();
    let image_char_width: u32 = image_pixel_width/char_width;
    
    let mut organised_image_pixels: Vec<Vec<Vec<[f64; 3]>>> = vec![vec![vec![[0.0, 0.0, 0.0]; 0]; image_char_width as usize]; image_char_height as usize];

    for y in 0..image.height() {
        for x in 0..image.width() {
            organised_image_pixels[
                (y as f64 / (image.height() as f64 / image_char_height as f64)).min(image_char_height as f64-1.0) as usize
            ][
                (x as f64 / (image.width() as f64 / image_char_width as f64)).min(image_char_width as f64-1.0) as usize
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

    let mut print_pixels: Vec<Vec<[f64; 3]>> = vec![vec![[0.0; 3]; image_char_width as usize]; image_char_height as usize];

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

            print_pixels[y as usize][x as usize] = sum_pixel;
        }
    }

    let mut buffer = String::new();
    for y in 0..print_pixels.len() {
        for x in 0..print_pixels[0].len() {
            let old_pixel = print_pixels[y][x];
            let (new_pixel, quant_error) = find_closest_colour(&old_pixel[0..3]);
        
            if x != 0 && y != 0 && x != print_pixels[0].len() - 1 && y != print_pixels.len() - 1 {
                print_pixels[y as usize][1 + x as usize][0] += quant_error[0] * 7.0 / 16.0;
                print_pixels[y as usize][1 + x as usize][1] += quant_error[1] * 7.0 / 16.0;
                print_pixels[y as usize][1 + x as usize][2] += quant_error[2] * 7.0 / 16.0;
            
                print_pixels[1 + y as usize][x as usize - 1][0] += quant_error[0] * 3.0 / 16.0;
                print_pixels[1 + y as usize][x as usize - 1][1] += quant_error[1] * 3.0 / 16.0;
                print_pixels[1 + y as usize][x as usize - 1][2] += quant_error[2] * 3.0 / 16.0;
            
                print_pixels[1 + y as usize][x as usize][0] += quant_error[0] * 5.0 / 16.0;
                print_pixels[1 + y as usize][x as usize][1] += quant_error[1] * 5.0 / 16.0;
                print_pixels[1 + y as usize][x as usize][2] += quant_error[2] * 5.0 / 16.0;
            
                print_pixels[1 + y as usize][1 + x as usize][0] += quant_error[0] * 1.0 / 16.0;
                print_pixels[1 + y as usize][1 + x as usize][1] += quant_error[1] * 1.0 / 16.0;
                print_pixels[1 + y as usize][1 + x as usize][2] += quant_error[2] * 1.0 / 16.0;
            }
            buffer.push_str(&format!("\x1b[48;5;{}m ", new_pixel));
        }
        buffer.push_str("\x1b[0m\n");
    }
    print!("{}", buffer);
    //
    //let img = image::open("/Users/codyryall/Desktop/code.png").expect("File not found!");
    //
    //let mut image_pixels = vec![vec![[255 as f64; 4]; img.width() as usize]; img.height() as usize];
    //
    //
    //let ratio = (terminal_pixel_size.0 as f64 / terminal_size.0 as f64) / (terminal_pixel_size.1 as f64 / terminal_size.1 as f64);
    //println!("{:?}", ratio);
    //
    //let WIDTH = (HEIGHT as f64 / ratio) as usize;
    ////get pixel dimensions per character of image being printed; it has a known height but its width is based on ratio
    //let char_height: f64 = img.height() as f64 / HEIGHT as f64;
    //let char_width: f64 = char_height / ratio;
    //let mut organised_pixels: Vec<Vec<Vec<[f64; 3]>>> = vec![vec![Vec::new(); WIDTH]; HEIGHT];
    //let mut pixels: Vec<Vec<[f64; 3]>> = vec![vec![[255 as f64; 3]; WIDTH]; HEIGHT];
    //
    //for y in 0..img.height() as usize {
    //    for x in 0..img.width() as usize {
    //        let pixel = img.get_pixel(x as u32, y as u32).0;
    //        organised_pixels[y/HEIGHT][x/WIDTH].push([
    //            pixel[0] as f64,
    //            pixel[1] as f64,
    //            pixel[2] as f64
    //        ]);
    //        //pixels[y][x] = [pixel[0] as f64, pixel[1] as f64, pixel[2] as f64, 255.0];
    //    }
    //}
    //
    ////make each colour in pixels the average of the colour lists in organised_pixels
    //for y in 0..pixels.len() {
    //    for x in 0..pixels[0].len() {
    //        let mut_pixels = &organised_pixels[y][x];
    //        let mut avg_colour: [f64; 3] = [0.0; 3];
    //        for colour in mut_pixels {
    //            avg_colour[0] += colour[0];
    //            avg_colour[1] += colour[1];
    //            avg_colour[2] += colour[2];
    //        }
    //        pixels[y][x] = [
    //            avg_colour[0] / mut_pixels.len() as f64,
    //            avg_colour[1] / mut_pixels.len() as f64,
    //            avg_colour[2] / mut_pixels.len() as f64
    //        ];
    //    }
    //}
    //
    //let mut buffer = String::new();
    //for y in 0..pixels.len() {
    //    for x in 0..pixels[0].len() {
    //        let old_pixel = *pixels.get(y as usize).unwrap().get(x as usize).unwrap();
    //        let (new_pixel, quant_error) = find_closest_colour(&old_pixel[0..3]);
    //    
    //        if x != 0 && y != 0 && x != pixels[0].len() - 1 && y != pixels.len() - 1 {
    //            pixels[y as usize][1 + x as usize][0] += quant_error[0] * 7.0 / 16.0;
    //            pixels[y as usize][1 + x as usize][1] += quant_error[1] * 7.0 / 16.0;
    //            pixels[y as usize][1 + x as usize][2] += quant_error[2] * 7.0 / 16.0;
    //        
    //            pixels[1 + y as usize][x as usize - 1][0] += quant_error[0] * 3.0 / 16.0;
    //            pixels[1 + y as usize][x as usize - 1][1] += quant_error[1] * 3.0 / 16.0;
    //            pixels[1 + y as usize][x as usize - 1][2] += quant_error[2] * 3.0 / 16.0;
    //        
    //            pixels[1 + y as usize][x as usize][0] += quant_error[0] * 5.0 / 16.0;
    //            pixels[1 + y as usize][x as usize][1] += quant_error[1] * 5.0 / 16.0;
    //            pixels[1 + y as usize][x as usize][2] += quant_error[2] * 5.0 / 16.0;
    //        
    //            pixels[1 + y as usize][1 + x as usize][0] += quant_error[0] * 1.0 / 16.0;
    //            pixels[1 + y as usize][1 + x as usize][1] += quant_error[1] * 1.0 / 16.0;
    //            pixels[1 + y as usize][1 + x as usize][2] += quant_error[2] * 1.0 / 16.0;
    //        }
    //        buffer.push_str(&format!("\x1b[48;5;{}m ", new_pixel));
    //    }
    //    buffer.push_str("\x1b[0m\n");
    //}
    //println!("{}", buffer);
}
