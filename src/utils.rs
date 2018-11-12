use nuklear::Color;

pub fn color_from_hex(rgb: u32) -> Color {
    Color {
        r: ((rgb & 0xff0000) >> 2 * 8) as u8,
        g: ((rgb & 0x00ff00) >> 1 * 8) as u8,
        b: (rgb & 0x0000ff) as u8,
        a: 100u8,
    }
}
