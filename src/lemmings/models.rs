////////////////////////////////////////////////////////////////////////////////
/// Levels

#[derive(Default, Debug)]
pub struct Skills {
    pub climbers: u16, // 2 bytes each, only lower byte is used, max 0x00FA
    pub floaters: u16,
    pub bombers: u16,
    pub blockers: u16,
    pub builders: u16,
    pub bashers: u16,
    pub miners: u16,
    pub diggers: u16,
}

#[derive(Default, Debug)]
pub struct Globals {
    pub release_rate: u16, // 0x0000 is slowest, 0x00FA is fastest
    pub num_of_lemmings: u16, // maximum 0x0072
    pub num_to_rescue: u16, // should be less than or equal to number of lemmings
    pub time_limit: u16, // max 0x00FF, 0x0001 to 0x0009 works best
    pub skills: Skills, // How many of each skill you get to start with.
    pub start_screen_xpos: u16, // 0x0000 to 0x04F0.  is rounded to nearest multiple of 8.
    pub normal_graphic_set: u16, // 0x0000 is dirt, 0x0001 is fire, 0x0002 is squasher, 0x0003 is pillar, 0x0004 is crystal, 0x0005 is brick, 0x0006 is rock, 0x0007 is snow and 0x0008 is bubble.
    pub extended_graphic_set: u16, // Apparently ignored in windows version.
    pub unknown: u16,
}

#[derive(Debug)]
pub enum ObjectModifier {
    Normal, // Draw full graphic, 0
    MustHaveTerrainUnderneathToBeVisible, // 40
    DoNotOverwriteExistingTerrain, // 80
}

impl ObjectModifier {
    #[inline]
    pub fn is_do_not_overwrite_existing_terrain(&self) -> bool {
        if let ObjectModifier::DoNotOverwriteExistingTerrain = self { true } else { false }
    }

    #[inline]
    pub fn is_must_have_terrain_underneath_to_be_visible(&self) -> bool {
        if let ObjectModifier::MustHaveTerrainUnderneathToBeVisible = self { true } else { false }
    }
}

#[derive(Debug)]
pub struct Object {
    pub x: i32, // Normalised to 0.
        // In file:
        // min 0xFFF8, max 0x0638.  0xFFF8 = -24, 0x0001 = -16, 0x0008 = -8,
        // 0x0010 = 0, 0x0018 = 8, ... , 0x0638 = 1576
        // note: should be multiples of 8

    pub y: i32, // Normalised.
        // In file:
        // min 0xFFD7, max 0x009F.  0xFFD7 = -41, 0xFFF8 = -8, 0xFFFF = -1,
        // 0x0000 = 0, ... , 0x009F = 159.  
        // note: can be any value in the specified range

    pub obj_id: usize, // min 0x0000, max 0x000F.  the object id is different in each
        // graphics set, however 0x0000 is always an exit and 0x0001 is always a start.  

    pub modifier: ObjectModifier, // can be 80 (do not overwrite existing terrain) or 40
	   // (must have terrain underneath to be visible). 00 specifies always
	   // draw full graphic.

    pub is_upside_down: bool, // can be 8F (display graphic upside-down) or 0F (display graphic normally)
}

#[derive(Debug)]
pub struct Terrain {
    pub do_not_overwrite_existing_terrain: bool,
    pub is_upside_down: bool,
    pub remove_terrain: bool,
    pub x: i32, // Normalised.
        // In file: min 0x0000, max 0x063F.  0x0000 = -16, 0x0008 = -8, 0x0010 = 0, 0x063f = 1583.
    pub y: i32, // Normalised. 
        // In file: min 0xEF0, max 0x518.  0xEF0 = -38, 0xEF8 = -37,
        // 0x020 = 0, 0x028 = 1, 0x030 = 2, 0x038 = 3, ... , 0x518 = 159
    pub terrain_id: usize,
}

#[derive(Debug)]
pub struct SteelArea {
    pub x: isize, // Normalised.
        // In file: min 0x000, max 0xC78.  0x000 = -16, 0x008 = -12,
        // 0x010 = -8, 0x018 = -4, ... , 0xC78 = 1580.
        // note: each hex value represents 4 pixels.
    pub y: isize, // Normalised.
        // In file: min 0x00, max 0x27. 0x00 = 0, 0x01 = 4, 0x02 = 8, ... , 0x27 = 156
        // note: each hex value represents 4 pixels
    pub width: u8, // 0-F, each value represents 4 pixels, 0=4, 1=8, 7=32
    pub height: u8,
}

#[derive(Default, Debug)]
pub struct Level {
    pub globals: Globals,
    pub objects: Vec<Object>, // Up to 32
    pub terrain: Vec<Terrain>, // Up to 400
    pub steel: Vec<SteelArea>, // Up to 32
    pub name: String,
}

////////////////////////////////////////////////////////////////////////////////
/// Ground

#[derive(Default, Debug)]
pub struct ObjectInfo {
    pub animation_flags: u16,
    pub start_animation_frame_index: u8,
    pub end_animation_frame_index: u8,
    pub width: usize,
    pub height: usize,
    pub animation_frame_data_size: u16,
    pub mask_offset_from_image: u16,
    pub trigger_left: u16,
    pub trigger_top: u16,
    pub trigger_width: u8,
    pub trigger_height: u8,
    pub trigger_effect_id: u8,
    pub animation_frames_base_loc: u16,
    pub preview_image_index: u16,
    pub trap_sound_effect_id: u8,
}

impl ObjectInfo {
    pub fn is_valid(&self) -> bool {
        return self.width>0 && self.height>0;
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct TerrainInfo {
    pub width: usize,
    pub height: usize,
    pub image_loc: u16,
    pub mask_loc: u16,
}

impl TerrainInfo {
    pub fn is_valid(&self) -> bool {
        return self.width>0 && self.height>0;
    }
}

#[derive(Default)]
pub struct Palettes {
    pub ega_custom: [u8; 8],
    pub ega_standard: [u8; 8],
    pub ega_preview: [u8; 8],
    pub vga_custom: [u32; 8], // RGB Palette entries 8...15. Only 6 bits so 0x3f = 100%
    pub vga_standard: [u32; 8], // Doesn't seem to be used by the game.
    pub vga_preview: [u32; 8], // Always seems to match custom.
}

// Upgrades a 6-bit colour to 8, while still allowing 100% black and white.
fn colour_upgrade(six: u8) -> u8 {
    if six == 0 { 0 } else { (six << 2) + 3 }
}

// Converts 6-bit rgb to rgba.
fn abgr_from_lemmings_rgb(rgb: u32) -> u32 {
    let r6: u8 = (rgb >> 16) as u8;
    let g6: u8 = (rgb >> 8) as u8; // 'as u8' simply truncates the red bits.
    let b6: u8 = rgb as u8;
    let r8: u8 = colour_upgrade(r6);
    let g8: u8 = colour_upgrade(g6);
    let b8: u8 = colour_upgrade(b6);
    return (0xff << 24) + ((b8 as u32) << 16) + ((g8 as u32) << 8) + (r8 as u32);
}

fn abgr_from_rgb(rgb: u32) -> u32 {
    let r: u8 = (rgb >> 16) as u8;
    let g: u8 = (rgb >> 8) as u8; // 'as u8' simply truncates the red bits.
    let b: u8 = rgb as u8;
    return (0xff << 24) + ((b as u32) << 16) + ((g as u32) << 8) + (r as u32);
}

impl Palettes {
    // Converts the palette to 0xaabbggrr format to suit the 'image' crate.
    pub fn as_abgr(&self) -> [u32; 16] {
        return [
            abgr_from_lemmings_rgb(0x000000), // black.
            abgr_from_lemmings_rgb(0x101038), // blue, used for the lemmings' bodies.
            abgr_from_lemmings_rgb(0x002C00), // green, used for hair.
            abgr_from_lemmings_rgb(0x3C3434), // white, used for skin.
            abgr_from_lemmings_rgb(0x2C2C00), // dirty yellow, used in the skill panel.
            abgr_from_lemmings_rgb(0x3C0808), // red, used in the nuke icon.
            abgr_from_lemmings_rgb(0x202020), // gray, used in the skill panel.
            abgr_from_rgb(self.vga_custom[0]), // Game duplicates custom[0] twice, oddly.
            abgr_from_rgb(self.vga_custom[0]),
            abgr_from_rgb(self.vga_custom[1]),
            abgr_from_rgb(self.vga_custom[2]),
            abgr_from_rgb(self.vga_custom[3]),
            abgr_from_rgb(self.vga_custom[4]),
            abgr_from_rgb(self.vga_custom[5]),
            abgr_from_rgb(self.vga_custom[6]),
            abgr_from_rgb(self.vga_custom[7]),
        ];
    }
}

pub struct Ground {
    pub object_info: [ObjectInfo; 16],
    pub terrain_info: [TerrainInfo; 64],
    pub palettes: Palettes,
}
