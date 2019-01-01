// This is for decoding the contents of main.dat
// https://www.camanis.net/lemmings/files/docs/lemmings_main_dat_file_format.txt

use std::io;
use std::io::Error;
use std::io::ErrorKind;
use super::helpers::BitsIterMS;
use lemmings::models::*;

// Creates a bit iterator from [u8].
macro_rules! iterate_bits { ($data:expr) => { $data.iter().flat_map(BitsIterMS::new); } }

impl Image {
    /// Parses where 0=transparent, 1=white.
    fn parse_1bpp(data: &[u8], width: usize, height: usize) -> Image {
        let pixels = width * height;
        let mut plane = iterate_bits!(data);
        let mut bitmap: Vec<u32> = Vec::with_capacity(pixels);
        for _ in 0..pixels {
            let bit = plane.next().unwrap();
            bitmap.push(if bit==0 { 0 } else { 0xffffffff } );
        }
        return Image { bitmap: bitmap, width: width, height: height };
    }

    fn parse_2bpp(data: &[u8], width: usize, height: usize, palette: [u32; 16]) -> Image {
        let pixels = width * height;
        let mut plane_0 = iterate_bits!(data);
        let mut plane_1 = iterate_bits!(data).skip(pixels);
        let mut bitmap: Vec<u32> = Vec::with_capacity(pixels);
        for _ in 0..pixels {
            let colour_index =
                plane_0.next().unwrap() +
                (plane_1.next().unwrap() << 1);
            let colour = palette[colour_index as usize];
            bitmap.push(colour);
        }
        return Image { bitmap: bitmap, width: width, height: height };
    }

    fn parse_3bpp(data: &[u8], width: usize, height: usize, palette: [u32; 16]) -> Image {
        let pixels = width * height;
        let mut plane_0 = iterate_bits!(data);
        let mut plane_1 = iterate_bits!(data).skip(pixels);
        let mut plane_2 = iterate_bits!(data).skip(pixels * 2);
        let mut bitmap: Vec<u32> = Vec::with_capacity(pixels);
        for _ in 0..pixels {
            let colour_index =
                plane_0.next().unwrap() +
                (plane_1.next().unwrap() << 1) +
                (plane_2.next().unwrap() << 2);
            let colour = palette[colour_index as usize];
            bitmap.push(colour);
        }
        return Image { bitmap: bitmap, width: width, height: height };
    }

    fn parse_4bpp(data: &[u8], width: usize, height: usize, palette: [u32; 16]) -> Image {
        let pixels = width * height;
        let mut plane_0 = iterate_bits!(data);
        let mut plane_1 = iterate_bits!(data).skip(pixels);
        let mut plane_2 = iterate_bits!(data).skip(pixels * 2);
        let mut plane_3 = iterate_bits!(data).skip(pixels * 3);
        let mut bitmap: Vec<u32> = Vec::with_capacity(pixels);
        for _ in 0..pixels {
            let colour_index =
                plane_0.next().unwrap() +
                (plane_1.next().unwrap() << 1) +
                (plane_2.next().unwrap() << 2) +
                (plane_3.next().unwrap() << 3);
            let colour = palette[colour_index as usize];
            bitmap.push(colour);
        }
        return Image { bitmap: bitmap, width: width, height: height };
    }
}

impl Animation {
    fn parse_2bpp(data: &[u8], frame_count: usize, width: usize, height: usize, palette: [u32; 16]) -> Animation {
        const BPP: usize = 2;
        let pixels = width * height;
        let mut frames: Vec<Vec<u32>> = Vec::with_capacity(frame_count);
        for frame_index in 0..frame_count {
            let offset_bits = frame_index * pixels * BPP;
            let mut plane_0 = iterate_bits!(data).skip(offset_bits);
            let mut plane_1 = iterate_bits!(data).skip(offset_bits + pixels);
            let mut bitmap: Vec<u32> = Vec::with_capacity(pixels);
            for _ in 0..pixels {
                let colour_index =
                    plane_0.next().unwrap() +
                    (plane_1.next().unwrap() << 1);
                let colour = palette[colour_index as usize];
                bitmap.push(colour);
            }
            frames.push(bitmap);
        }
        return Animation { frames: frames, width: width, height: height };
    }

    fn parse_3bpp(data: &[u8], frame_count: usize, width: usize, height: usize, palette: [u32; 16]) -> Animation {
        const BPP: usize = 3;
        let pixels = width * height;
        let mut frames: Vec<Vec<u32>> = Vec::with_capacity(frame_count);
        for frame_index in 0..frame_count {
            let offset_bits = frame_index * pixels * BPP;
            let mut plane_0 = iterate_bits!(data).skip(offset_bits);
            let mut plane_1 = iterate_bits!(data).skip(offset_bits + pixels);
            let mut plane_2 = iterate_bits!(data).skip(offset_bits + pixels * 2);
            let mut bitmap: Vec<u32> = Vec::with_capacity(pixels);
            for _ in 0..pixels {
                let colour_index =
                    plane_0.next().unwrap() +
                    (plane_1.next().unwrap() << 1) +
                    (plane_2.next().unwrap() << 2);
                let colour = palette[colour_index as usize];
                bitmap.push(colour);
            }
            frames.push(bitmap);
        }
        return Animation { frames: frames, width: width, height: height };
    }

    fn parse_4bpp(data: &[u8], frame_count: usize, width: usize, height: usize, palette: [u32; 16]) -> Animation {
        const BPP: usize = 4;
        let pixels = width * height;
        let mut frames: Vec<Vec<u32>> = Vec::with_capacity(frame_count);
        for frame_index in 0..frame_count {
            let offset_bits = frame_index * pixels * BPP;
            let mut plane_0 = iterate_bits!(data).skip(offset_bits);
            let mut plane_1 = iterate_bits!(data).skip(offset_bits + pixels);
            let mut plane_2 = iterate_bits!(data).skip(offset_bits + pixels * 2);
            let mut plane_3 = iterate_bits!(data).skip(offset_bits + pixels * 3);
            let mut bitmap: Vec<u32> = Vec::with_capacity(pixels);
            for _ in 0..pixels {
                let colour_index =
                    plane_0.next().unwrap() +
                    (plane_1.next().unwrap() << 1) +
                    (plane_2.next().unwrap() << 2) +
                    (plane_3.next().unwrap() << 3);
                let colour = palette[colour_index as usize];
                bitmap.push(colour);
            }
            frames.push(bitmap);
        }
        return Animation { frames: frames, width: width, height: height };
    }

    fn parse(data: &[u8], frames: usize, width: usize, height: usize, palette: [u32; 16], bpp: u8) -> io::Result<Animation> {
        if bpp == 2 {
            Ok(Animation::parse_2bpp(data, frames, width, height, palette))
        } else if bpp == 3 {
            Ok(Animation::parse_3bpp(data, frames, width, height, palette))
        } else if bpp == 4 {
            Ok(Animation::parse_4bpp(data, frames, width, height, palette))
        } else {
            Err(Error::new(ErrorKind::InvalidInput, "Unsupported BPP"))
        }
    }
}

impl LemmingAnimations {
    fn parse(data: &[u8], palette: [u32; 16]) -> io::Result<LemmingAnimations> {
        Ok(LemmingAnimations {
            walking_right: Animation::parse(&data[0x0000..], 8, 16, 10, palette, 2)?,
            jumping_right: Animation::parse(&data[0x0140..], 1, 16, 10, palette, 2)?,
            walking_left: Animation::parse(&data[0x0168..], 8, 16, 10, palette, 2)?,
            jumping_left: Animation::parse(&data[0x02A8..], 1, 16, 10, palette, 2)?,
            digging: Animation::parse(&data[0x02D0..], 16, 16, 14, palette, 3)?,
            climbing_right: Animation::parse(&data[0x0810..], 8, 16, 12, palette, 2)?,
            climbing_left: Animation::parse(&data[0x0990..], 8, 16, 12, palette, 2)?,
            drowning: Animation::parse(&data[0x0B10..], 16, 16, 10, palette, 2)?,
            post_climb_right: Animation::parse(&data[0x0D90..], 8, 16, 12, palette, 2)?,
            post_climb_left: Animation::parse(&data[0x0F10..], 8, 16, 12, palette, 2)?,
            brick_laying_right: Animation::parse(&data[0x1090..], 16, 16, 13, palette, 3)?,
            brick_laying_left: Animation::parse(&data[0x1570..], 16, 16, 13, palette, 3)?, 
            bashing_right: Animation::parse(&data[0x1A50..], 32, 16, 10, palette, 3)?, 
            bashing_left: Animation::parse(&data[0x21D0..], 32, 16, 10, palette, 3)?, 
            mining_right: Animation::parse(&data[0x2950..], 24, 16, 13, palette, 3)?, 
            mining_left: Animation::parse(&data[0x30A0..], 24, 16, 13, palette, 3)?, 
            falling_right: Animation::parse(&data[0x37F0..], 4, 16, 10, palette, 2)?, 
            falling_left: Animation::parse(&data[0x3890..], 4, 16, 10, palette, 2)?, 
            pre_umbrella_right: Animation::parse(&data[0x3930..], 4, 16, 16, palette, 3)?,
            umbrella_right: Animation::parse(&data[0x3AB0..], 4, 16, 16, palette, 3)?, 
            pre_umbrella_left: Animation::parse(&data[0x3C30..], 4, 16, 16, palette, 3)?, 
            umbrella_left: Animation::parse(&data[0x3DB0..], 4, 16, 16, palette, 3)?,
            splatting: Animation::parse(&data[0x3F30..], 16, 16, 10, palette, 2)?, 
            exiting: Animation::parse(&data[0x41B0..], 8, 16, 13, palette, 2)?, 
            fried: Animation::parse(&data[0x4350 ..], 14, 16, 14, palette, 4)?, 
            blocking: Animation::parse(&data[0x4970..], 16, 16, 10, palette, 2)?, 
            shrugging_right: Animation::parse(&data[0x4BF0..], 8, 16, 10, palette, 2)?, 
            shrugging_left: Animation::parse(&data[0x4D30..], 8, 16, 10, palette, 2)?, 
            oh_no_ing: Animation::parse(&data[0x4E70..], 16, 16, 10, palette, 2)?, 
            explosion: Animation::parse(&data[0x50F0..], 1, 32, 32, palette, 3)?,
        })
    }
}

impl Mask {
    fn parse(data: &[u8], frame_count: usize, width: usize, height: usize) -> Mask {
        let pixels = width * height;
        let mut frames: Vec<Vec<u8>> = Vec::with_capacity(frame_count);
        for frame_index in 0..frame_count {
            let offset_bits = frame_index * pixels;
            let mut plane = iterate_bits!(data).skip(offset_bits);
            let mut bitmap: Vec<u8> = Vec::with_capacity(pixels);
            for _ in 0..pixels {
                let bit = plane.next().unwrap();
                bitmap.push(bit);
            }
            frames.push(bitmap);
        }
        return Mask { frames: frames, width: width, height: height };
    }
}

impl Masks {
    fn parse(data: &[u8]) -> Masks {
        Masks {
            bash_right: Mask::parse(&data[0x0000..], 4, 16, 10),
            bash_left:  Mask::parse(&data[0x0050..], 4, 16, 10),
            mine_right: Mask::parse(&data[0x00a0..], 2, 16, 13),
            mine_left:  Mask::parse(&data[0x00d4..], 2, 16, 13),
            explosion:  Mask::parse(&data[0x0108..], 1, 16, 22),
        }
    }
}

fn parse_countdown_numbers(data: &[u8]) -> [Image; 10] {
    [
        Image::parse_1bpp(&data[0x017C..], 8, 8),
        Image::parse_1bpp(&data[0x0174..], 8, 8),
        Image::parse_1bpp(&data[0x016C..], 8, 8),
        Image::parse_1bpp(&data[0x0164..], 8, 8),
        Image::parse_1bpp(&data[0x015C..], 8, 8),
        Image::parse_1bpp(&data[0x0154..], 8, 8),
        Image::parse_1bpp(&data[0x014C..], 8, 8),
        Image::parse_1bpp(&data[0x0144..], 8, 8),
        Image::parse_1bpp(&data[0x013C..], 8, 8),
        Image::parse_1bpp(&data[0x0134..], 8, 8),
    ]
}

impl SkillNumberDigits {
    fn parse(data: &[u8]) -> SkillNumberDigits {
        SkillNumberDigits {
            left: [
                Image::parse_1bpp(&data[0x1908..], 8, 8),
                Image::parse_1bpp(&data[0x1918..], 8, 8),
                Image::parse_1bpp(&data[0x1928..], 8, 8),
                Image::parse_1bpp(&data[0x1938..], 8, 8),
                Image::parse_1bpp(&data[0x1948..], 8, 8),
                Image::parse_1bpp(&data[0x1958..], 8, 8),
                Image::parse_1bpp(&data[0x1968..], 8, 8),
                Image::parse_1bpp(&data[0x1978..], 8, 8),
                Image::parse_1bpp(&data[0x1988..], 8, 8),
                Image::parse_1bpp(&data[0x1998..], 8, 8),
            ],
            right: [
                Image::parse_1bpp(&data[0x1900..], 8, 8),
                Image::parse_1bpp(&data[0x1910..], 8, 8),
                Image::parse_1bpp(&data[0x1920..], 8, 8),
                Image::parse_1bpp(&data[0x1930..], 8, 8),
                Image::parse_1bpp(&data[0x1940..], 8, 8),
                Image::parse_1bpp(&data[0x1950..], 8, 8),
                Image::parse_1bpp(&data[0x1960..], 8, 8),
                Image::parse_1bpp(&data[0x1970..], 8, 8),
                Image::parse_1bpp(&data[0x1980..], 8, 8),
                Image::parse_1bpp(&data[0x1990..], 8, 8),
            ]
        }
    }
}

impl GameFont {
    fn parse(data: &[u8], palette: [u32; 16]) -> GameFont {
        const SIZE_PER_CHAR: usize = 0x30;
        let mut font: GameFont = Default::default();
        let mut offset: usize = 0;
        font.percent = Image::parse_3bpp(&data[offset..], 8, 16, palette);
        offset += SIZE_PER_CHAR;
        for i in 0..10 {
            font.digits[i] = Image::parse_3bpp(&data[offset..], 8, 16, palette);
            offset += SIZE_PER_CHAR;
        }
        font.dash = Image::parse_3bpp(&data[offset..], 8, 16, palette);
        offset += SIZE_PER_CHAR;
        for i in 0..26 {
            font.letters[i] = Image::parse_3bpp(&data[offset..], 8, 16, palette);
            offset += SIZE_PER_CHAR;
        }
        return font;
    }
}

impl MenuFont {
    fn parse(data: &[u8], palette: [u32; 16]) -> MenuFont {
        const SIZE_PER_CHAR: usize = 0x60;
        let mut font: MenuFont = Default::default();
        let mut offset: usize = 0;
        for _ in 0..94 {
            let image = Image::parse_3bpp(&data[offset..], 16, 16, palette);
            font.characters.push(image);
            offset += SIZE_PER_CHAR;
        }
        return font;
    }
}

impl MainMenu {
    fn parse(section_3: &[u8], section_4: &[u8], palette: [u32; 16]) -> MainMenu {
        let mut back_palette = palette; // Make 0 solid black, not transparent, for the background.
        back_palette[0] = 0xff000000;
        MainMenu {
            background:     Image::parse_2bpp(&section_3, 320, 104, back_palette),
            logo:           Image::parse_4bpp(&section_3[0x2080..], 632, 94, palette),
            f1:             Image::parse_4bpp(&section_3[0x9488..], 120, 61, palette),
            f2:             Image::parse_4bpp(&section_3[0xa2d4..], 120, 61, palette),
            f3:             Image::parse_4bpp(&section_3[0xb120..], 120, 61, palette),
            f4:             Image::parse_4bpp(&section_3[0xdc04..], 120, 61, palette),
            level_rating:   Image::parse_4bpp(&section_3[0xbf6c..], 120, 61, palette),
            exit_to_dos:    Image::parse_4bpp(&section_3[0xCDB8..], 120, 61, palette),
            music_note:     Image::parse_4bpp(&section_3[0xEA50..], 64, 31, palette),
            fx:             Image::parse_4bpp(&section_3[0xEE30..], 64, 31, palette),
            blink1:         Animation::parse_4bpp(&section_4[0x0000..], 8, 32, 12, palette),
            blink2:         Animation::parse_4bpp(&section_4[0x0600..], 8, 32, 12, palette),
            blink3:         Animation::parse_4bpp(&section_4[0x0C00..], 8, 32, 12, palette),
            blink4:         Animation::parse_4bpp(&section_4[0x1200..], 8, 32, 12, palette),
            blink5:         Animation::parse_4bpp(&section_4[0x1800..], 8, 32, 12, palette),
            blink6:         Animation::parse_4bpp(&section_4[0x1E00..], 8, 32, 12, palette),
            blink7:         Animation::parse_4bpp(&section_4[0x2400..], 8, 32, 12, palette),
            left_scroller:  Animation::parse_4bpp(&section_4[0x2A00..], 16, 48, 16, palette),
            right_scroller: Animation::parse_4bpp(&section_4[0x4200..], 16, 48, 16, palette),
            reel:           Image::parse_4bpp(&section_4[0x5A00..], 16, 16, palette),
            mayhem:         Image::parse_4bpp(&section_4[0x5A80..], 16, 16, palette),
            taxing:         Image::parse_4bpp(&section_4[0x5E4C..], 16, 16, palette),
            tricky:         Image::parse_4bpp(&section_4[0x6218..], 16, 16, palette),
            fun:            Image::parse_4bpp(&section_4[0x65E4..], 16, 16, palette),
            menu_font:      MenuFont::parse(&section_4[0x69B0..], palette)
        }
    }
}

macro_rules! abgr_from_rgb { ($r:expr, $g:expr, $b:expr) => {
    0xff000000 + (($b as u32) << 16) + (($g as u32) << 8) + ($r as u32)
}}

pub fn parse(sections: &Vec<Vec<u8>>) -> io::Result<MainDat> {
    if sections.len() < 7 {
        return Err(Error::new(ErrorKind::InvalidData, "Not enough sections"))
    }

    let menu_palette: [u32; 16] = [
        0, // Transparent black.
        abgr_from_rgb!(128, 64, 32), // Browns 
        abgr_from_rgb!( 96, 48, 32), // 
        abgr_from_rgb!( 48,  0, 16), //
        abgr_from_rgb!( 32,  8,124), // Purples 
        abgr_from_rgb!( 64, 44,144), //
        abgr_from_rgb!(104, 88,164), // 
        abgr_from_rgb!(152,140,188), // 
        abgr_from_rgb!(  0, 80,  0), // Greens
        abgr_from_rgb!(  0, 96, 16), //
        abgr_from_rgb!(  0,112, 32), //
        abgr_from_rgb!(  0,128, 64), //
        abgr_from_rgb!(208,208,208), // White 
        abgr_from_rgb!(176,176,  0), // Yellow 
        abgr_from_rgb!( 64, 80,176), // Blue 
        abgr_from_rgb!(224,128,144), // Pink  
    ];
    let mut game_palette = menu_palette;
    game_palette[1] = abgr_from_rgb!( 64, 64,224); // Blue
    game_palette[2] = abgr_from_rgb!(  0,176,  0); // Green
    game_palette[3] = abgr_from_rgb!(240,208,208); // White
    game_palette[4] = abgr_from_rgb!(176,176,  0); // Yellow
    game_palette[5] = abgr_from_rgb!(240, 32, 32); // Red
    game_palette[6] = abgr_from_rgb!(128,128,128); // Grey

    Ok(MainDat {
        lemming_animations: LemmingAnimations::parse(&sections[0], game_palette)?,
        masks: Masks::parse(&sections[1]),
        countdown_numbers: parse_countdown_numbers(&sections[1]),
        skill_panel_high_perf: Image::parse_4bpp(&sections[2], 320, 40, game_palette),
        skill_number_digits: SkillNumberDigits::parse(&sections[2]),
        game_font_high_perf: GameFont::parse(&sections[2][0x19a0..], game_palette),
        main_menu: MainMenu::parse(&sections[3], &sections[4], menu_palette),
        skill_panel: Image::parse_4bpp(&sections[6], 320, 40, game_palette),
        game_font: GameFont::parse(&sections[6][0x1900..], game_palette),
    })
}
