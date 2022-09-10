use image;
use image::GenericImageView; // to allow calling .pixels()

mod colours;
use colours::*;

fn find_closest_colour(clr: [u8; 4]) -> usize {
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
    closest
}

fn main() {
    let img = image::open("/Users/codyryall/Desktop/code.png").expect("File not found!");
    let mut buffer = String::new();
    for x in 0..img.width() {
        
        for y in 0..img.height() {
            let clr = img.get_pixel(x, y);
            let clr = find_closest_colour(clr.0);
            buffer.push_str(&format!("\x1b[48;5;{}m ", clr));
        }
        buffer.push_str("\x1b[0m\n");
    }
    println!("{}", buffer);
}
