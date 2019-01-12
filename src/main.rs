#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use nuklear::{Color, Context, Flags, nk_string};
use nuklear as nk;
use nuklear_backend_gdi::*;

use winreg::RegKey;

use itertools::Itertools;

mod utils;
mod keyboard;

use crate::keyboard as kb;

fn main() {
	let mappings = match load_registry() {
		Ok(mappings) => mappings,
		Err(err) => panic!("Error loading registry: {}", err), //TODO error handling (message to user?)
	};

	let mut state = State {
		window_size: (800, 600),
		pairs: mappings.into_iter().map(|(key1, key2)| (Some(key1), Some(key2))).collect_vec(),
	};

	let mut allo = nk::Allocator::new_vec();
	let (mut dr, mut ctx, font) = bundle(
		"Nuklear Rust GDI Demo",
		state.window_size.0 as u16, state.window_size.1 as u16,
		"Segoe UI", 16,
		&mut allo,
	);
	let clear_color: Color = utils::color_from_hex(0xc47fef);

	loop {
		if !dr.process_events(&mut ctx) {
			break;
		}

		state.window_size = utils::get_window_size(dr.window().unwrap());

		ctx.style_set_font(dr.font_by_id(font).unwrap());

		layout(&mut ctx, &mut dr, &mut state);
		dr.render(&mut ctx, clear_color);
	}
}

struct State {
	window_size: (i32, i32),
	pairs: Vec<(Option<kb::Key>, Option<kb::Key>)>,
}

fn layout(ctx: &mut Context, _dr: &mut Drawer, state: &mut State) {
	if !ctx.begin(
		nk_string!("kb_remapper"),
		nk::Rect { x: 0.0f32, y: 0.0f32, w: state.window_size.0 as f32, h: state.window_size.1 as f32 },
		0 as Flags,
	) {
		panic!("ctx.begin returned false");
	}

	ctx.layout_row_dynamic(30.0, 10);
	for key in kb::KEYS {
		if ctx.button_text(key.label) {
			on_key_button_press(state, key.clone());
		}
	}

	let mut pair_to_delete = None;
	for (idx, pair) in state.pairs.iter().enumerate() {
		let key1 = match &pair.0 {
			Some(k) => k.label,
			None => "",
		};
		let key2 = match &pair.1 {
			Some(k) => k.label,
			None => "",
		};

		ctx.layout_row_dynamic(30.0, 4);
		ctx.text(key1, nk::TextAlignment::NK_TEXT_LEFT as Flags);
		ctx.text("->", nk::TextAlignment::NK_TEXT_LEFT as Flags);
		ctx.text(key2, nk::TextAlignment::NK_TEXT_LEFT as Flags);

		if ctx.button_text("[X]") {
			pair_to_delete = Some(idx);
			break;
		}
	}
	if let Some(idx) = pair_to_delete {
		state.pairs.remove(idx);
	}

	ctx.layout_row_dynamic(30.0, 1);
	if ctx.button_text("Apply changes") {
		apply_registry_changes(state);
	}

	ctx.end();
}

fn on_key_button_press(state: &mut State, key: kb::Key) {
	match state.pairs.last_mut() {
		Some(pair) => match pair.1 {
			Some(_) => state.pairs.push((Some(key.clone()), None)),
			None => if pair.0.as_ref().unwrap() != &key { pair.1 = Some(key.clone()) } else {},
		}
		None => state.pairs.push((Some(key.clone()), None)),
	}
}

fn apply_registry_changes(state: &mut State) {}


fn load_registry() -> Result<Vec<(kb::Key, kb::Key)>, std::io::Error> {
	let hklm = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
	let kb_layout = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Keyboard Layout")?;
	let scancode_map = match kb_layout.get_raw_value("Scancode Map") {
		Ok(map) => map.bytes,
		Err(_) => return Ok(vec![]),
	};

	let keys = scancode_map.iter()
		.skip(12)
		.map(|x| *x as u32)
		.chunks(4)
		.into_iter()
		.map(|chunk| chunk.collect::<Vec<_>>())
		.filter_map(|mapping| {
			let scan_code1 = mapping[0] | (mapping[1] << 8);
			let scan_code2 = mapping[2] | (mapping[3] << 8);

			let (mut key1, mut key2) = (None, None);
			for key in kb::KEYS {
				if key.scan_code == scan_code1 {
					key1 = Some(key.clone());
					if key2.is_some() { break; }
				}
				if key.scan_code == scan_code2 {
					key2 = Some(key.clone());
					if key1.is_some() { break; }
				}
			}

			if key1.is_some() && key2.is_some() {
				return Some((key1.unwrap(), key2.unwrap()));
			} else {
				return None;
			}
		}).collect_vec();

	Ok(keys)
}

