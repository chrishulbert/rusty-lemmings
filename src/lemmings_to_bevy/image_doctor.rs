// This doctors some of the lemmings images, eg remove the F1 F2 etc from the menu options.

use crate::lemmings::models::Image;

// Turn transparent to black, and remove the main colour.
pub fn doctor_skill(original: &Image) -> Image {
    let mut doctored = original.clone();
    let to_remove = doctored.bitmap[0];
    for i in 0..doctored.bitmap.len() {
        let c = doctored.bitmap[i];
        if c == to_remove {
            doctored.bitmap[i] = 0; // Transparent.
        } else if (doctored.bitmap[i] as u8) == 0 {
            doctored.bitmap[i] = 0xff; // Black.
        }
    }
    doctored
}

pub fn doctor_clear_to_black(original: &Image) -> Image {
    let mut doctored = original.clone();
    for i in 0..doctored.bitmap.len() {
        let a = doctored.bitmap[i] as u8;
        if a == 0 {
            doctored.bitmap[i] = 0xff; // Black.
        }
    }
    doctored
}

pub fn doctor_f1(original: &Image) -> Image {
    let mut doctored = original.clone();
    remove_rect(7, 26, 94, 25, &mut doctored, RectRemovalSource::TopRight); // Main area.
    remove_rect(7, 19, 21, 8, &mut doctored, RectRemovalSource::TopRight); // Most of F1.
    remove_rect(9, 18, 17, 1, &mut doctored, RectRemovalSource::TopRight); // Top of F1 with clear.
    remove_rect(6, 18, 13, 7, &mut doctored, RectRemovalSource::TopRight); // Make a shadow around the hand.
    remove_rect(6, 18, 12, 6, &mut doctored, RectRemovalSource::TopLeft); // Fill in the hand.
    doctored
}

pub fn doctor_f2(original: &Image) -> Image {
    let mut doctored = original.clone();
    remove_rect(23, 27, 74, 27, &mut doctored, RectRemovalSource::TopRight); // Main area.
    remove_rect(8, 19, 21, 11, &mut doctored, RectRemovalSource::TopRight); // F2.
    remove_rect(9, 18, 19, 1, &mut doctored, RectRemovalSource::TopRight); // Top of F2.
    remove_rect(8, 19, 1, 1, &mut doctored, RectRemovalSource::TopLeft); // Finish the top left triangle.
    remove_rect(8, 20, 1, 2, &mut doctored, RectRemovalSource::BottomLeft); // Border around triangle.
    remove_rect(9, 19, 1, 2, &mut doctored, RectRemovalSource::BottomLeft); // Border around triangle.
    doctored
}

pub fn doctor_f3(original: &Image) -> Image {
    let mut doctored = original.clone();
    remove_rect(7, 21, 21, 10, &mut doctored, RectRemovalSource::TopRight); // Most of F3.
    remove_rect(8, 20, 20, 1, &mut doctored, RectRemovalSource::TopRight); // Second top row.
    remove_rect(9, 19, 20, 1, &mut doctored, RectRemovalSource::TopRight); // Top row.
    remove_rect(7, 21, 1, 3, &mut doctored, RectRemovalSource::BottomLeftDown1); // Outline of hand.
    remove_rect(8, 20, 1, 1, &mut doctored, RectRemovalSource::BottomLeftDown1); // Outline of hand.
    remove_rect(9, 19, 9, 1, &mut doctored, RectRemovalSource::BottomLeftDown1); // Outline of hand.
    doctored
}

enum RectRemovalSource {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomLeftDown1,
}


// Picks up the colour from the top right + 1.
fn remove_rect(x: usize, y: usize, width: usize, height: usize, image: &mut Image, source: RectRemovalSource) {
    let fill: u32 = match source {
        RectRemovalSource::TopRight => image.bitmap[y * image.width + x + width],
        RectRemovalSource::TopLeft => image.bitmap[y * image.width + x - 1],
        RectRemovalSource::BottomLeft => image.bitmap[(y + height - 1) * image.width + x - 1],
        RectRemovalSource::BottomLeftDown1 => image.bitmap[(y + height) * image.width + x - 1],
    };
    for inside_y in 0..height {
        let mut offset = (y + inside_y) * image.width + x;
        for _inside_x in 0..width {
            image.bitmap[offset] = fill;
            offset += 1;
        }
    }
}