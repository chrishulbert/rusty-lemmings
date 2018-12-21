// This is for parsing lemmings LVL files.
// https://www.camanis.net/lemmings/files/docs/lemmings_lvl_file_format.txt

use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::slice;

#[derive(Default)]
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

#[derive(Default)]
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

pub enum ObjectModifier {
    Normal, // Draw full graphic, 0
    MustHaveTerrainUnderneathToBeVisible, // 40
    DoNotOverwriteExistingTerrain, // 80
}

impl ObjectModifier {
    fn from_lvl(lvl: u8) -> ObjectModifier {
        if lvl == 0x80 {
            return ObjectModifier::DoNotOverwriteExistingTerrain;
        } else if lvl == 0x40 {
            return ObjectModifier::MustHaveTerrainUnderneathToBeVisible;
        } else {
            return ObjectModifier::Normal;
        }
    }

    #[inline]
    pub fn is_do_not_overwrite_existing_terrain(&self) -> bool {
        if let ObjectModifier::DoNotOverwriteExistingTerrain = self { true } else { false }
    }

    #[inline]
    pub fn is_must_have_terrain_underneath_to_be_visible(&self) -> bool {
        if let ObjectModifier::MustHaveTerrainUnderneathToBeVisible = self { true } else { false }
    }
}

pub struct Object {
    pub x: isize, // Normalised to 0.
        // In file:
        // min 0xFFF8, max 0x0638.  0xFFF8 = -24, 0x0001 = -16, 0x0008 = -8,
        // 0x0010 = 0, 0x0018 = 8, ... , 0x0638 = 1576
        // note: should be multiples of 8

    pub y: isize, // Normalised.
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

#[derive(Default)]
pub struct Level {
    pub globals: Globals,
    pub objects: Vec<Object>, // Up to 32
    pub terrain: Vec<Terrain>, // Up to 400
    pub steel: Vec<SteelArea>, // Up to 32
    pub name: String,
}

fn string_from_vec(vec: Vec<u8>) -> io::Result<String> {
    match String::from_utf8(vec).ok() {
        Some(t) => Ok(t),
        None => Err(Error::new(ErrorKind::InvalidData, "Bad string")),
    }
}

// Exposes the 'next' as a result so you can use '?'.
fn read_u8(data: &mut slice::Iter<u8>) -> io::Result<u8> {
    match data.next() {
        Some(t) => Ok(*t),
        None => Err(Error::new(ErrorKind::UnexpectedEof, "No data remaining")),
    }
}

// Unlike the GROUND file format, WORDs in LVL are stored big-endian (camanis.net).
fn read_u16(data: &mut slice::Iter<u8>) -> io::Result<u16> {
    let big = read_u8(data)?;
    let little = read_u8(data)?;
    Ok(((big as u16) << 8) + (little as u16))
}

/// Decompresses all the sections from a compressed dat file.
/// Returns a vec of sections. Each section is a vec of its data.
pub fn parse(data: &[u8]) -> io::Result<Level> {
    if data.len() != 2048 {
        return Err(Error::new(ErrorKind::InvalidData, "Wrong length"))
    }
    let mut level: Level = Default::default();
    let mut data_iter = data.into_iter();

    // Globals.
    level.globals.release_rate = read_u16(&mut data_iter)?;
    level.globals.num_of_lemmings = read_u16(&mut data_iter)?;
    level.globals.num_to_rescue = read_u16(&mut data_iter)?;
    level.globals.time_limit = read_u16(&mut data_iter)?;
    level.globals.skills.climbers = read_u16(&mut data_iter)?;
    level.globals.skills.floaters = read_u16(&mut data_iter)?;
    level.globals.skills.bombers = read_u16(&mut data_iter)?;
    level.globals.skills.blockers = read_u16(&mut data_iter)?;
    level.globals.skills.builders = read_u16(&mut data_iter)?;
    level.globals.skills.bashers = read_u16(&mut data_iter)?;
    level.globals.skills.miners = read_u16(&mut data_iter)?;
    level.globals.skills.diggers = read_u16(&mut data_iter)?;
    level.globals.start_screen_xpos = read_u16(&mut data_iter)?;
    level.globals.normal_graphic_set = read_u16(&mut data_iter)?;
    level.globals.extended_graphic_set = read_u16(&mut data_iter)?;
    let _unused = read_u16(&mut data_iter)?;

    // Objects.
    for _ in 0..32 {
        let ix = read_u16(&mut data_iter)? as i16; // Will convert eg 0xfff8 to -24;
        let iy = read_u16(&mut data_iter)? as i16;
        let id = read_u16(&mut data_iter)?;
        let ma = read_u8(&mut data_iter)?;
        let mb = read_u8(&mut data_iter)?;
        let is_bad = (ix==0 && iy==0 && id==0) || id>=16;
        if !is_bad {
            level.objects.push(Object {
                x: ix as isize,
                y: iy as isize + 4,
                obj_id: id as usize,
                modifier: ObjectModifier::from_lvl(ma),
                is_upside_down: mb == 0x8f,
            });
        }
    }

    // Terrain.
    for _ in 0..400 {
        let a = read_u8(&mut data_iter)?; // significant nibble = flags, other = x.
        let b = read_u8(&mut data_iter)?; // x. 
        let c = read_u8(&mut data_iter)?; // First 8 of 9 bits of y.
        let d = read_u8(&mut data_iter)?; // Another bit of y, and terrain id.
        let terrain_id = d & 0x7f;
        let is_bad = (a==0xff && b==0xff && c==0xff && d==0xff) || terrain_id>=64;
        if !is_bad {
            let x: u16 = (((a & 0xf) as u16) << 8) + (b as u16);
            let y_bits: u16 = ((c as u16) << 1) + ((d >> 7) as u16);
            let y_2s_comp: u16 = if y_bits & 0x100 == 0 { y_bits } else { y_bits | 0xfe00 };
            let y_i: i16 = y_2s_comp as i16;
            level.terrain.push(Terrain {
                do_not_overwrite_existing_terrain: ((a >> 4) & 8) == 8,
                is_upside_down: ((a >> 4) & 4) == 4,
                remove_terrain: ((a >> 4) & 2) == 2,
                x: x as isize,
                y: y_i as isize,
                terrain_id: terrain_id as usize,
            });
        }
    }

    // Steel.
    for _ in 0..32 {
        let a = read_u8(&mut data_iter)?; // First 8 of 9 bits of x.
        let b = read_u8(&mut data_iter)?; // Last x bit, y.
        let c = read_u8(&mut data_iter)?; // Area.
        let d = read_u8(&mut data_iter)?; // Unused, pity after all that packing!
        let is_bad = a==0 && b==0 && c==0 && d==0;
        if !is_bad {
            let x: u16 = ((a as u16) << 1) + ((b >> 7) as u16);
            let y: u8 = b & 0x7f;
            level.steel.push(SteelArea {
                x: (x as isize),
                y: (y as isize) * 4,
                width: c >> 4,
                height: c & 0xf,
            });
        }
    }

    // Name.
    let mut str_raw: Vec<u8> = Vec::with_capacity(32);
    for _ in 0..32 {
        let byte = read_u8(&mut data_iter)?;
        str_raw.push(byte);
    }
    let raw_name = string_from_vec(str_raw)?;
    level.name = raw_name.trim().to_string();

    Ok(level)
}
