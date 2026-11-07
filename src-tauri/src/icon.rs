//! Draws the tray badge: a rounded square with up to two pixel-font glyphs, sized to the tray.

/// 3x5 glyphs, one row per byte (bit 2 = left column): digits 0-9, then 'h'.
const FONT: [[u8; 5]; 11] = [
    [7, 5, 5, 5, 7], [2, 6, 2, 2, 7], [7, 1, 7, 4, 7], [7, 1, 7, 1, 7], [5, 5, 7, 1, 1],
    [7, 4, 7, 1, 7], [7, 4, 7, 5, 7], [7, 1, 2, 2, 2], [7, 5, 7, 5, 7], [7, 5, 7, 1, 7],
    [4, 4, 7, 5, 5],
];

/// Badge colors: background and glyphs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Look {
    pub bg: [u8; 3],
    pub fg: [u8; 3],
}

/// RGBA pixels of a `size` x `size` badge showing `text` (chars from "0123456789h", at most 2).
pub fn render(text: &str, look: Look, size: u32) -> Vec<u8> {
    let s = size as usize;
    let mut px = vec![0u8; s * s * 4];
    let r = size as f32 / 4.0;
    for (i, p) in px.chunks_exact_mut(4).enumerate() {
        p.copy_from_slice(&[look.bg[0], look.bg[1], look.bg[2], corner_alpha(i % s, i / s, size as f32, r)]);
    }
    let k = ((size - 2) / 7).min((size - 4) / 5).max(1) as usize;
    let glyphs: Vec<&[u8; 5]> = text
        .chars()
        .map(|c| &FONT[if c == 'h' { 10 } else { c.to_digit(10).expect("badge text is digits or 'h'") as usize }])
        .collect();
    let w = glyphs.len() * 4 * k - k;
    let (x0, y0) = ((s - w) / 2, (s - 5 * k) / 2);
    for (g, glyph) in glyphs.iter().enumerate() {
        for (row, bits) in glyph.iter().enumerate() {
            for col in (0..3).filter(|c| bits >> (2 - c) & 1 == 1) {
                fill(&mut px, s, x0 + (g * 4 + col) * k, y0 + row * k, k, look.fg);
            }
        }
    }
    px
}

/// Opacity of pixel (x, y) inside a rounded square of side `s` and corner radius `r`, anti-aliased.
fn corner_alpha(x: usize, y: usize, s: f32, r: f32) -> u8 {
    let edge = |v: usize| (r - (v as f32 + 0.5)).max(v as f32 + 0.5 - (s - r)).max(0.0);
    let d = edge(x).hypot(edge(y));
    ((r + 0.5 - d).clamp(0.0, 1.0) * 255.0) as u8
}

/// Paints a k x k opaque block at (x, y).
fn fill(px: &mut [u8], s: usize, x: usize, y: usize, k: usize, c: [u8; 3]) {
    for yy in y..y + k {
        for xx in x..x + k {
            px[(yy * s + xx) * 4..][..4].copy_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
}
