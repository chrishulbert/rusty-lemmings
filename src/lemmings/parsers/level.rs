// This is for parsing lemmings LVL files.
// https://www.camanis.net/lemmings/files/docs/lemmings_lvl_file_format.txt

use std::io::{Error, ErrorKind, Result};
use std::slice::Iter;

use crate::lemmings::models::*;

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
}

fn string_from_vec(vec: Vec<u8>) -> Result<String> {
    match String::from_utf8(vec).ok() {
        Some(t) => Ok(t),
        None => Err(Error::new(ErrorKind::InvalidData, "Bad string")),
    }
}

// Exposes the 'next' as a result so you can use '?'.
fn read_u8(data: &mut Iter<u8>) -> Result<u8> {
    match data.next() {
        Some(t) => Ok(*t),
        None => Err(Error::new(ErrorKind::UnexpectedEof, "No data remaining")),
    }
}

// Unlike the GROUND file format, WORDs in LVL are stored big-endian (camanis.net).
fn read_u16(data: &mut Iter<u8>) -> Result<u16> {
    let big = read_u8(data)?;
    let little = read_u8(data)?;
    Ok(((big as u16) << 8) + (little as u16))
}

/// Decompresses all the sections from a compressed dat file.
/// Returns a vec of sections. Each section is a vec of its data.
pub fn parse(data: &[u8]) -> Result<Level> {
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
                x: ix as i32,
                y: iy as i32,
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
            let flags: u8 = a >> 4;
            let do_not_overwrite_existing_terrain = (flags & 8) == 8;
            let remove_terrain = (flags & 2) == 2;
            level.terrain.push(Terrain {
                do_not_overwrite_existing_terrain: do_not_overwrite_existing_terrain,
                is_upside_down: (flags & 4) == 4,
                remove_terrain: remove_terrain && !do_not_overwrite_existing_terrain, // If both flags are on, only honor 'do not overwrite'.
                x: x as isize,
                y: y_i as isize - 4,
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
                y: (y as isize) * 4 - 4, // TODO test this makes the steel in the correct place?
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
