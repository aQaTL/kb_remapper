use nuklear::Color;

pub fn color_from_hex(rgb: u32) -> Color {
	Color {
		r: ((rgb & 0xff0000) >> 2 * 8) as u8,
		g: ((rgb & 0x00ff00) >> 1 * 8) as u8,
		b: (rgb & 0x0000ff) as u8,
		a: 100u8,
	}
}

use winapi::shared::windef::{HWND, RECT};
use winapi::um::winuser::{MessageBoxW, MB_ICONERROR};

pub fn get_window_size(hwnd: HWND) -> (i32, i32) {
	unsafe {
		let mut rect = RECT {
			left: 0,
			top: 0,
			right: 0,
			bottom: 0,
		};
		winapi::um::winuser::GetClientRect(hwnd, &mut rect as *mut RECT);
		return (rect.right - rect.left, rect.bottom - rect.top);
	}
}

pub fn panic_with_message_box(err: impl std::error::Error, hwnd: Option<HWND>) {
	unsafe {
		MessageBoxW(hwnd.unwrap_or(std::ptr::null_mut()),
					err.to_string()
						.encode_utf16()
						.chain("\0".encode_utf16())
						.collect::<Vec<_>>().as_ptr(),
					"Panic\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
					MB_ICONERROR);
	}
	panic!("{:?}", err);
}