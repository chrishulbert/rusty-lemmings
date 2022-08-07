// This doctors some of the lemmings images, eg remove the F1 F2 etc from the menu options.

use crate::lemmings::models::Image;

pub fn doctor_f1(original: &Image) -> Image {
    let mut doctored = original.clone();
    remove_rect(7, 26, 94, 25, &mut doctored, RectRemovalSource::TopRight); // Main area.
    remove_rect(7, 19, 21, 8, &mut doctored, RectRemovalSource::TopRight); // Most of F1.
    remove_rect(9, 18, 17, 1, &mut doctored, RectRemovalSource::TopRight); // Top of F1 with clear.
    remove_rect(6, 18, 13, 7, &mut doctored, RectRemovalSource::TopRight); // Make a shadow around the hand.
    remove_rect(6, 18, 12, 6, &mut doctored, RectRemovalSource::TopLeft); // Fill in the hand.
    doctored
}

enum RectRemovalSource {
    TopRight,
    TopLeft,
}

// Picks up the colour from the top right + 1.
fn remove_rect(x: usize, y: usize, width: usize, height: usize, image: &mut Image, source: RectRemovalSource) {
    let fill: u32 = match source {
        RectRemovalSource::TopLeft => image.bitmap[y * image.width + x - 1],
        RectRemovalSource::TopRight => image.bitmap[y * image.width + x + width],
    };
    for inside_y in 0..height {
        let mut offset = (y + inside_y) * image.width + x;
        for _inside_x in 0..width {
            image.bitmap[offset] = fill;
            offset += 1;
        }
    }
}