use winreg::RegKey;

use itertools::Itertools;
use byteorder::{LittleEndian, ReadBytesExt};

use std::io::Read;
use std::io::Error;

use super::{*};

type KeyMapping = (Option<Key>, Option<Key>);

pub fn load_key_mappings() -> Result<Vec<KeyMapping>, Error> {
	let hklm = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
	let super_layout = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Keyboard Layout")?;
	let scancode_map = super_layout.get_raw_value("Scancode Map")?.bytes;

	let mut rdr = std::io::Cursor::new(scancode_map);
	rdr.set_position(4 * 2); //Skips headers

	let entries_count = rdr.read_u32::<LittleEndian>()? - 1; // last entry is always NULL
	let mut entries = vec![0u8; (entries_count * 4) as usize];
	rdr.read(&mut entries)?;

	let mut rdr = std::io::Cursor::new(entries);
	Ok((0..entries_count * 2) //Every entry is a pair of 2 scancodes
		.map(|_| rdr.read_u16::<LittleEndian>().unwrap())
		.map(|scancode| KEYS.iter().find(|key| key.scan_code == scancode).map(Clone::clone))
		.tuples::<(_, _)>()
		.collect_vec())
}

pub fn save_key_mappings(mappings: &Vec<KeyMapping>) -> Result<(), ()> {




	Ok(())
}
