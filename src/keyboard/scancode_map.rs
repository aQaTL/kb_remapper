use winreg::RegKey;

use itertools::Itertools;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::io::Read;
use std::io::Error;

use super::{*};

/// The order of items is (remapped_key, new_scancode)
/// If Key is None, it's scancode will be 0
type KeyMapping = (Option<Key>, Option<Key>);

pub fn load_key_mappings() -> Result<Vec<KeyMapping>, Error> {
	let hklm = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
	let keyboard_layout = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Keyboard Layout")?;
	let scancode_map = match keyboard_layout.get_raw_value("Scancode Map") {
		Ok(value) => value.bytes,
		Err(_) => return Ok(vec![]),
	};

	let mut rdr = std::io::Cursor::new(scancode_map);
	rdr.set_position(4 * 2); //Skips version and flag headers

	let entries_count = rdr.read_u32::<LittleEndian>()? - 1; // last entry is always NULL
	let mut entries = vec![0u8; (entries_count * 4) as usize];
	rdr.read(&mut entries)?;

	let mut rdr = std::io::Cursor::new(entries);
	Ok((0..entries_count * 2) //Every entry is a pair of 2 scancodes
		.map(|_| rdr.read_u16::<LittleEndian>().unwrap())
		.map(|scancode| KEYS.iter().find(|key| key.scan_code == scancode).map(Clone::clone))
		.tuples::<(_, _)>()
		.map(|(x, y)| (y, x)) //Will be in (remapped_key, new_scancode) order
		.collect_vec())
}

pub fn save_key_mappings(mappings: &Vec<KeyMapping>) -> Result<(), Error> {
	let mut wrt = Vec::with_capacity(4 * 3 + (mappings.len() + 1) * 4);
	wrt.append(&mut vec![0u8; 4 * 2]);
	wrt.write_u32::<LittleEndian>((mappings.len() + 1) as u32)?;
	//Windows wants keys in (new_scancode, remapped_key) order
	for (key2, key1) in mappings.iter().chain(&[(None, None)]) {
		wrt.write_u16::<LittleEndian>(key1.as_ref().map_or(0u16, |k| k.scan_code))?;
		wrt.write_u16::<LittleEndian>(key2.as_ref().map_or(0u16, |k| k.scan_code))?;
	}

	let hklm = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
	let (keyboard_layout, _) = hklm.create_subkey(r"SYSTEM\CurrentControlSet\Control\Keyboard Layout")?;
	let reg_value = winreg::RegValue{ bytes: wrt, vtype: winreg::enums::RegType::REG_BINARY };
	keyboard_layout.set_raw_value("Scancode Map", &reg_value)?;

	Ok(())
}
