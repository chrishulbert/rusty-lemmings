use std::collections::HashMap;
use std::vec::IntoIter;

////////////////////////////////////////////////////////////////////////////////
/// Levels

#[derive(Default, Debug, Clone)]
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

#[derive(Default, Debug, Clone)]
pub struct Globals {
    pub release_rate: u16, // 0x0000 is slowest, 0x00FA is fastest
    pub num_of_lemmings: u16, // maximum 0x0072
    pub num_to_rescue: u16, // should be less than or equal to number of lemmings
    pub time_limit: u16, // max 0x00FF, 0x0001 to 0x0009 works best
    pub skills: Skills, // How many of each skill you get to start with.
    pub start_screen_xpos: u16, // 0x0000 to 0x04F0.  is rounded to nearest multiple of 8.
    pub normal_graphic_set: u16, // AKA ground. 0x0000 is dirt, 0x0001 is fire, 0x0002 is squasher, 0x0003 is pillar, 0x0004 is crystal, 0x0005 is brick, 0x0006 is rock, 0x0007 is snow and 0x0008 is bubble.
    pub extended_graphic_set: u16, // Apparently ignored in windows version.
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Terrain {
    pub do_not_overwrite_existing_terrain: bool,
    pub is_upside_down: bool,
    pub remove_terrain: bool,
    pub x: isize, // Normalised.
        // In file: min 0x0000, max 0x063F.  0x0000 = -16, 0x0008 = -8, 0x0010 = 0, 0x063f = 1583.
    pub y: isize, // Normalised. 
        // In file: min 0xEF0, max 0x518.  0xEF0 = -38, 0xEF8 = -37,
        // 0x020 = 0, 0x028 = 1, 0x030 = 2, 0x038 = 3, ... , 0x518 = 159
    pub terrain_id: usize,
}

#[derive(Debug, Clone)]
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

#[derive(Default, Debug, Clone)]
pub struct Level {
    pub globals: Globals,
    pub objects: Vec<Object>, // Up to 32
    pub terrain: Vec<Terrain>, // Up to 400
    pub steel: Vec<SteelArea>, // Up to 32
    pub name: String,
}

////////////////////////////////////////////////////////////////////////////////
/// Ground

#[derive(Default, Debug, Clone)]
pub struct ObjectInfo {
    pub is_exit: bool, // According to lemmings_lvl_file_format.txt, the first object for any ground is always exit, second is entrance.
    pub is_entrance: bool, 
    pub animation_flags: u16, // bit 0 = 0 for loops, 1 for triggered animations.
    pub start_animation_frame_index: u8, 
    pub frame_count: u8, // aka end_animation_frame_index in the docs, but I suspect that's wrong, because if you +1 to get the frame count, it fails to load.
    pub width: usize,
    pub height: usize,
    pub animation_frame_data_size: u16,
    pub mask_offset_from_image: u16,
    pub trigger_left: u16,
    pub trigger_top: u16,
    pub trigger_width: u8,
    pub trigger_height: u8,
    pub trigger_effect_id: u8, // 0=none, 1=lemming exits, 4=trigger trap, 5=drown, 6=disintegrate, 7=one way wall left, 8=one way right, 9=steel
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

#[derive(Default, Clone)]
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
fn rgba_from_lemmings_rgb(rgb: u32) -> u32 {
    let r6: u8 = (rgb >> 16) as u8;
    let g6: u8 = (rgb >> 8) as u8; // 'as u8' simply truncates the red bits.
    let b6: u8 = rgb as u8;
    let r8: u8 = colour_upgrade(r6);
    let g8: u8 = colour_upgrade(g6);
    let b8: u8 = colour_upgrade(b6);
    return ((r8 as u32) << 24) + ((g8 as u32) << 16) + ((b8 as u32) << 8) + 0xff;
}

fn rgba_from_rgb(rgb: u32) -> u32 {
    let r: u8 = (rgb >> 16) as u8;
    let g: u8 = (rgb >> 8) as u8; // 'as u8' simply truncates the red bits.
    let b: u8 = rgb as u8;
    return ((r as u32) << 24) + ((g as u32) << 16) + ((b as u32) << 8) + 0xff;
}

impl Palettes {
    // Converts the palette to 0xaabbggrr format to suit the 'image' crate.
    pub fn as_rgba(&self) -> [u32; 16] {
        return [
            rgba_from_lemmings_rgb(0x000000), // black.
            rgba_from_lemmings_rgb(0x101038), // blue, used for the lemmings' bodies.
            rgba_from_lemmings_rgb(0x002C00), // green, used for hair.
            rgba_from_lemmings_rgb(0x3C3434), // white, used for skin.
            rgba_from_lemmings_rgb(0x2C2C00), // dirty yellow, used in the skill panel.
            rgba_from_lemmings_rgb(0x3C0808), // red, used in the nuke icon.
            rgba_from_lemmings_rgb(0x202020), // gray, used in the skill panel.
            rgba_from_rgb(self.vga_custom[0]), // Game duplicates custom[0] twice, oddly.
            rgba_from_rgb(self.vga_custom[0]),
            rgba_from_rgb(self.vga_custom[1]),
            rgba_from_rgb(self.vga_custom[2]),
            rgba_from_rgb(self.vga_custom[3]),
            rgba_from_rgb(self.vga_custom[4]),
            rgba_from_rgb(self.vga_custom[5]),
            rgba_from_rgb(self.vga_custom[6]),
            rgba_from_rgb(self.vga_custom[7]),
        ];
    }
}

#[derive(Clone)]
pub struct Ground {
    pub object_info: [ObjectInfo; 16],
    pub terrain_info: [TerrainInfo; 64],
    pub palettes: Palettes,
}

////////////////////////////////////////////////////////////////////////////////
/// Images

#[derive(Default, Clone)]
pub struct Image {
    pub bitmap: Vec<u32>,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone)]
pub struct Animation {
    pub frames: Vec<Vec<u32>>, // Think of this as an array of frames, where each frame is Vec<u32>.
    pub width: usize,
    pub height: usize,
}

#[derive(Clone)]
pub struct Mask {
    pub frames: Vec<Vec<u8>>, // 1 means take a pixel out, 0 means leave alone.
    pub width: isize,
    pub height: isize,
}

////////////////////////////////////////////////////////////////////////////////
/// Main dat

#[derive(Clone)]
pub struct LemmingAnimations {
    pub walking_right: Animation,
    pub jumping_right: Animation, // Walking up a step 3-6px tall. This is a 1-frame 'animation'.
    pub walking_left: Animation,
    pub jumping_left: Animation, // This is a 1-frame 'animation'.
    pub digging: Animation,
    pub climbing_right: Animation,
    pub climbing_left: Animation,
    pub drowning: Animation,
    pub post_climb_right: Animation,
    pub post_climb_left: Animation,
    pub brick_laying_right: Animation,
    pub brick_laying_left: Animation,
    pub bashing_right: Animation,
    pub bashing_left: Animation,
    pub mining_right: Animation,
    pub mining_left: Animation,
    pub falling_right: Animation,
    pub falling_left: Animation,
    pub pre_umbrella_right: Animation,
    pub umbrella_right: Animation,
    pub pre_umbrella_left: Animation,
    pub umbrella_left: Animation,
    pub splatting: Animation,
    pub exiting: Animation,
    pub fried: Animation,
    pub blocking: Animation,
    pub shrugging_right: Animation, // Builder running out of bricks.
    pub shrugging_left: Animation,
    pub oh_no_ing: Animation,
    pub explosion: Animation, // 1 frame.
}

#[derive(Clone)]
pub struct Masks {
    pub bash_right: Mask,
    pub bash_left: Mask,
    pub mine_right: Mask,
    pub mine_left: Mask,
    pub explosion: Mask,
}

#[derive(Clone)]
pub struct SkillNumberDigits {
    pub left: [Image; 10],
    pub right: [Image; 10],
}

#[derive(Default, Clone)]
pub struct GameFont {
    pub percent: Image,
    pub digits: [Image; 10], // 0-9
    pub dash: Image,
    pub letters: [Image; 26], // A-Z
}

#[derive(Clone)]
pub struct MainMenu {
    pub background: Image,
    pub logo: Image,
    pub f1: Image,
    pub f2: Image,
    pub f3: Image,
    pub f4: Image,
    pub level_rating: Image,
    pub exit_to_dos: Image,
    pub music_note: Image,
    pub fx: Image,

    pub blink1: Animation,
    pub blink2: Animation,
    pub blink3: Animation,
    pub blink4: Animation,
    pub blink5: Animation,
    pub blink6: Animation,
    pub blink7: Animation,
    pub left_scroller: Animation,
    pub right_scroller: Animation,
    pub reel: Image,
    // TODO change this to skill indexes so it's not lemmings-1-specific.
    // TODO change to support 5 skills for oh-no-more.
    pub mayhem: Image,
    pub taxing: Image,
    pub tricky: Image,
    pub fun: Image,
    pub menu_font: Animation, // 16x16, 94 frames, '!'(33) - '~'(126), in ascii order. Not really an animation, but this makes texture atlas conversion simpler. 
}

#[derive(Clone)]
pub struct MainDat {
    pub lemming_animations: LemmingAnimations,
    pub masks: Masks,
    pub countdown_numbers: [Image; 10],
    pub skill_panel_high_perf: Image,
    pub skill_number_digits: SkillNumberDigits,
    pub game_font_high_perf: GameFont,
    pub main_menu: MainMenu,
    pub skill_panel: Image,
    pub skill_selection: Image,
    pub game_font: GameFont,
    pub mouse_cursor: Image,
    pub mouse_cursor_hovering: Image,
}

////////////////////////////////////////////////////////////////////////////////
/// Loader

pub type ImageMap = HashMap<i32, Image>;
pub type AnimationMap = HashMap<i32, Animation>;

#[derive(Clone)]
pub struct GroundCombined {
    pub ground: Ground, 
    pub terrain_sprites: ImageMap,
    pub object_sprites: AnimationMap,
}

pub type GroundMap = HashMap<i32, GroundCombined>;
pub type LevelMap = HashMap<i32, Level>; // Key is file# * 100 + section. Eg 203 = LEVEL002.DAT section 3.
pub type SpecialMap = HashMap<i32, Image>;

#[derive(Clone)]
pub struct Game {
    pub name: String, // Eg 'Oh No More Lemmings'
    pub id: String, // Eg 'ohnomore'
    pub path: String, // Eg '/Users/foo/Lemmings/ohnomore'
    pub levels: LevelMap,
    pub specials: SpecialMap,
    pub grounds: GroundMap,
    pub main: MainDat,
}

pub struct Games {
    pub lemmings: Option<Game>,
    pub oh_no_more: Option<Game>,
    pub christmas_91: Option<Game>,
    pub christmas_92: Option<Game>,
    pub holiday_93: Option<Game>,
    pub holiday_94: Option<Game>,
}

impl IntoIterator for Games {
    type Item = Game;
    type IntoIter = IntoIter<Game>;
    fn into_iter(self) -> Self::IntoIter {
        let mut v: Vec<Game> = Vec::new();
        if let Some(game) = self.lemmings     { v.push(game); }
        if let Some(game) = self.oh_no_more   { v.push(game); }
        if let Some(game) = self.christmas_91 { v.push(game); }
        if let Some(game) = self.christmas_92 { v.push(game); }
        if let Some(game) = self.holiday_93   { v.push(game); }
        if let Some(game) = self.holiday_94   { v.push(game); }
        v.into_iter()
    }
}

impl Games {
    pub fn as_vec(&self) -> Vec<&Game> {
        let mut v: Vec<&Game> = Vec::new();
        if let Some(ref game) = self.lemmings     { v.push(game); }
        if let Some(ref game) = self.oh_no_more   { v.push(game); }
        if let Some(ref game) = self.christmas_91 { v.push(game); }
        if let Some(ref game) = self.christmas_92 { v.push(game); }
        if let Some(ref game) = self.holiday_93   { v.push(game); }
        if let Some(ref game) = self.holiday_94   { v.push(game); }
        return v;
    }
}

/////////////////////////////////////////////////////////
/// For iterating through all the assets for pre-scaling. 
impl Game {
    pub fn level_named<'a>(&'a self, name: &str) -> Option<&'a Level> {
        for (_, l) in &self.levels {
            if l.name == name {
                return Some(l)
            }
        }
        None
    }
    
    pub fn all_assets(&self) -> Vec<AssetToPreProcess> {
        let mut all = Vec::<AssetToPreProcess>::new();        

        all.push(AssetToPreProcess{name: "lemming.walking_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.walking_right),});
        all.push(AssetToPreProcess{name: "lemming.jumping_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.jumping_right),});
        all.push(AssetToPreProcess{name: "lemming.walking_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.walking_left),});
        all.push(AssetToPreProcess{name: "lemming.jumping_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.jumping_left),});
        all.push(AssetToPreProcess{name: "lemming.digging".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.digging),});
        all.push(AssetToPreProcess{name: "lemming.climbing_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.climbing_right),});
        all.push(AssetToPreProcess{name: "lemming.climbing_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.climbing_left),});
        all.push(AssetToPreProcess{name: "lemming.drowning".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.drowning),});
        all.push(AssetToPreProcess{name: "lemming.post_climb_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.post_climb_right),});
        all.push(AssetToPreProcess{name: "lemming.post_climb_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.post_climb_left),});
        all.push(AssetToPreProcess{name: "lemming.brick_laying_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.brick_laying_right),});
        all.push(AssetToPreProcess{name: "lemming.brick_laying_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.brick_laying_left),});
        all.push(AssetToPreProcess{name: "lemming.bashing_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.bashing_right),});
        all.push(AssetToPreProcess{name: "lemming.bashing_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.bashing_left),});
        all.push(AssetToPreProcess{name: "lemming.mining_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.mining_right),});
        all.push(AssetToPreProcess{name: "lemming.mining_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.mining_left),});
        all.push(AssetToPreProcess{name: "lemming.falling_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.falling_right),});
        all.push(AssetToPreProcess{name: "lemming.falling_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.falling_left),});
        all.push(AssetToPreProcess{name: "lemming.pre_umbrella_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.pre_umbrella_right),});
        all.push(AssetToPreProcess{name: "lemming.umbrella_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.umbrella_right),});
        all.push(AssetToPreProcess{name: "lemming.pre_umbrella_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.pre_umbrella_left),});
        all.push(AssetToPreProcess{name: "lemming.umbrella_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.umbrella_left),});
        all.push(AssetToPreProcess{name: "lemming.splatting".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.splatting),});
        all.push(AssetToPreProcess{name: "lemming.exiting".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.exiting),});
        all.push(AssetToPreProcess{name: "lemming.fried".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.fried),});
        all.push(AssetToPreProcess{name: "lemming.blocking".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.blocking),});
        all.push(AssetToPreProcess{name: "lemming.shrugging_right".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.shrugging_right),});
        all.push(AssetToPreProcess{name: "lemming.shrugging_left".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.shrugging_left),});
        all.push(AssetToPreProcess{name: "lemming.oh_no_ing".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.oh_no_ing),});
        all.push(AssetToPreProcess{name: "lemming.explosion".to_string(), content: AnimationOrImage::Animation(&self.main.lemming_animations.explosion),});

        all.push(AssetToPreProcess{name: "font.percent".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.percent),});
        all.push(AssetToPreProcess{name: "font.dash".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.dash),});
        all.push(AssetToPreProcess{name: "font.0".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.digits[0]),});
        all.push(AssetToPreProcess{name: "font.1".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.digits[1]),});
        all.push(AssetToPreProcess{name: "font.2".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.digits[2]),});
        all.push(AssetToPreProcess{name: "font.3".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.digits[3]),});
        all.push(AssetToPreProcess{name: "font.4".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.digits[4]),});
        all.push(AssetToPreProcess{name: "font.5".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.digits[5]),});
        all.push(AssetToPreProcess{name: "font.6".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.digits[6]),});
        all.push(AssetToPreProcess{name: "font.7".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.digits[7]),});
        all.push(AssetToPreProcess{name: "font.8".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.digits[8]),});
        all.push(AssetToPreProcess{name: "font.9".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.digits[9]),});
        all.push(AssetToPreProcess{name: "font.a".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[0]),});
        all.push(AssetToPreProcess{name: "font.b".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[1]),});
        all.push(AssetToPreProcess{name: "font.c".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[2]),});
        all.push(AssetToPreProcess{name: "font.d".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[3]),});
        all.push(AssetToPreProcess{name: "font.e".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[4]),});
        all.push(AssetToPreProcess{name: "font.f".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[5]),});
        all.push(AssetToPreProcess{name: "font.g".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[6]),});
        all.push(AssetToPreProcess{name: "font.h".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[7]),});
        all.push(AssetToPreProcess{name: "font.i".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[8]),});
        all.push(AssetToPreProcess{name: "font.j".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[9]),});
        all.push(AssetToPreProcess{name: "font.k".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[10]),});
        all.push(AssetToPreProcess{name: "font.l".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[11]),});
        all.push(AssetToPreProcess{name: "font.m".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[12]),});
        all.push(AssetToPreProcess{name: "font.n".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[13]),});
        all.push(AssetToPreProcess{name: "font.o".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[14]),});
        all.push(AssetToPreProcess{name: "font.p".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[15]),});
        all.push(AssetToPreProcess{name: "font.q".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[16]),});
        all.push(AssetToPreProcess{name: "font.r".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[17]),});
        all.push(AssetToPreProcess{name: "font.s".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[18]),});
        all.push(AssetToPreProcess{name: "font.t".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[19]),});
        all.push(AssetToPreProcess{name: "font.u".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[20]),});
        all.push(AssetToPreProcess{name: "font.v".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[21]),});
        all.push(AssetToPreProcess{name: "font.w".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[22]),});
        all.push(AssetToPreProcess{name: "font.x".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[23]),});
        all.push(AssetToPreProcess{name: "font.y".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[24]),});
        all.push(AssetToPreProcess{name: "font.z".to_string(), content: AnimationOrImage::Image(&self.main.game_font_high_perf.letters[25]),});

        all
    }
}

pub struct AssetToPreProcess<'a> {
    pub name: String,
    pub content: AnimationOrImage<'a>,
}

pub enum AnimationOrImage<'a> {
    Animation(&'a Animation),
    Image(&'a Image),
}
