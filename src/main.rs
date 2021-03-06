#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use nuklear::{Color, Context, Flags, nk_string};
use nuklear as nk;
use nuklear_backend_gdi::*;

use std::io::ErrorKind;

mod utils;
mod keyboard;

use crate::keyboard as kb;

fn main() {
	let key_mappings = match kb::load_key_mappings() {
		Ok(km) => km,
		Err(err) => {
			utils::panic_with_message_box(err, None);
			vec![] //Calm down compiler
		}
	};
	let mut state = State {
		window_size: (800, 600),
		pairs: key_mappings,
		info: String::new(),
	};

	let mut allo = nk::Allocator::new_vec();
	let (mut dr, mut ctx, font) = bundle(
		"kb_remapper",
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
	info: String,
}

fn layout(ctx: &mut Context, dr: &mut Drawer, state: &mut State) {
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
		apply_registry_changes(state, dr);
	}

	if !state.info.is_empty() {
		ctx.layout_row_dynamic(20.0, 1);
		ctx.text(&state.info, nk::TextAlignment::NK_TEXT_CENTERED as Flags)
	}

	ctx.end();
}

fn on_key_button_press(state: &mut State, key: kb::Key) {
	match state.pairs.last_mut() {
		Some(pair) => match pair.1 {
			Some(_) if key.scan_code != 0 => state.pairs.push((Some(key.clone()), None)),
			None => if pair.0.as_ref().unwrap() != &key { pair.1 = Some(key.clone()) } else {},
			_ => (),
		}
		None if key.scan_code != 0 => state.pairs.push((Some(key.clone()), None)),
		_ => (),
	}
}


fn apply_registry_changes(state: &mut State, dr: &Drawer) {
	match kb::save_key_mappings(&state.pairs) {
		Ok(_) => state.info = String::from("Saved successfully. Reboot the computer to apply the changes."),
		Err(err) => match err.kind() {
			ErrorKind::PermissionDenied => {
				state.info = String::from("Permission denied. Run as administrator to make system changes.")
			}
			_ => utils::panic_with_message_box(err, Some(dr.window().unwrap())),
		},
	}
}
