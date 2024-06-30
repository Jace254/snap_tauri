use image::{ImageBuffer, Rgba};

// Convert ARGB to yuv420
pub fn argb_to_i420(image: &ImageBuffer<Rgba<u8>, Vec<u8>>, dest: &mut Vec<u8>) {
    let width = image.width() as usize;
    let height = image.height() as usize;

    dest.clear();

    // Y Plane
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x as u32, y as u32);
            let r = pixel[0] as i32;
            let g = pixel[1] as i32;
            let b = pixel[2] as i32;

            let y_value = (66 * r + 129 * g + 25 * b + 128) / 256 + 16;
            dest.push(clamp(y_value));
        }
    }

    // U Plane
    for y in (0..height).step_by(2) {
        for x in (0..width).step_by(2) {
            let pixel = image.get_pixel(x as u32, y as u32);
            let r = pixel[0] as i32;
            let g = pixel[1] as i32;
            let b = pixel[2] as i32;

            let u_value = (-38 * r - 74 * g + 112 * b + 128) / 256 + 128;
            dest.push(clamp(u_value));
        }
    }

    // V Plane
    for y in (0..height).step_by(2) {
        for x in (0..width).step_by(2) {
            let pixel = image.get_pixel(x as u32, y as u32);
            let r = pixel[0] as i32;
            let g = pixel[1] as i32;
            let b = pixel[2] as i32;

            let v_value = (112 * r - 94 * g - 18 * b + 128) / 256 + 128;
            dest.push(clamp(v_value));
        }
    }
}

fn clamp(x: i32) -> u8 {
    x.min(255).max(0) as u8
}
