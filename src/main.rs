extern crate winreg;

#[macro_use]
extern crate nuklear;
extern crate nuklear_backend_gdi;

extern crate image;
extern crate winapi;

use nuklear::{Color, Context, PanelFlags, Flags};
use nuklear as nk;
use nuklear_backend_gdi::*;

use winreg::RegKey;

mod utils;

static WINDOW_SIZE: (u16, u16) = (400, 600); //TODO make drawer window size public

fn main() {
    let names = load_registry();

    let mut allo = nk::Allocator::new_vec();
    let (mut dr, mut ctx, font) = bundle(
        "Nuklear Rust GDI Demo", WINDOW_SIZE.0, WINDOW_SIZE.1, "Segoe UI", 16, &mut allo,
    );
    let clear_color: Color = utils::color_from_hex(0xc47fef);

    let mut state = State {
        items: names,
    };

    loop {
        if !dr.process_events(&mut ctx) {
            break;
        }

        ctx.style_set_font(dr.font_by_id(font).unwrap());

        layout(&mut ctx, &mut dr, &mut state);
        dr.render(&mut ctx, clear_color);
    }
}

struct State {
    items: Vec<String>,
}

fn layout(ctx: &mut Context, dr: &mut Drawer, state: &mut State) {
    if !ctx.begin(
        nk_string!("kb_remapper"),
        nk::Rect { x: 0.0f32, y: 0.0f32, w: WINDOW_SIZE.0 as f32, h: WINDOW_SIZE.1 as f32 },
        0 as Flags,
    ) {
        panic!("ctx.begin returned false");
    }

    ctx.layout_row_dynamic(30.0, 1);
    if ctx.button_text("Blabla") {
        state.items.iter().for_each(|x| println!("{}", x));
    }

    ctx.end();
}


fn load_registry() -> Vec<String> {
    let hklm = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    hklm.enum_keys().into_iter().
        filter_map(|x| x.ok()).
        collect()
}

