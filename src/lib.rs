use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };

    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[inline]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#').unwrap_or(s);
        match s.len() {
            6 => {
                let v = u32::from_str_radix(s, 16).ok()?;
                Some(Self::rgb(
                    ((v >> 16) & 0xFF) as u8,
                    ((v >> 8) & 0xFF) as u8,
                    (v & 0xFF) as u8,
                ))
            }
            8 => {
                let v = u32::from_str_radix(s, 16).ok()?;
                Some(Self::rgba(
                    ((v >> 24) & 0xFF) as u8,
                    ((v >> 16) & 0xFF) as u8,
                    ((v >> 8) & 0xFF) as u8,
                    (v & 0xFF) as u8,
                ))
            }
            _ => None,
        }
    }

    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let s = 1.0 - t;
        Color {
            r: (self.r as f32 * s + other.r as f32 * t).round() as u8,
            g: (self.g as f32 * s + other.g as f32 * t).round() as u8,
            b: (self.b as f32 * s + other.b as f32 * t).round() as u8,
            a: (self.a as f32 * s + other.a as f32 * t).round() as u8,
        }
    }

    pub fn premultiply(&self) -> (u8, u8, u8) {
        let af = self.a as f32 / 255.0;
        (
            (self.r as f32 * af).round() as u8,
            (self.g as f32 * af).round() as u8,
            (self.b as f32 * af).round() as u8,
        )
    }

    #[inline]
    pub const fn pack_rgba(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }

    #[inline]
    pub const fn unpack_rgba(v: u32) -> Self {
        Self {
            r: ((v >> 24) & 0xFF) as u8,
            g: ((v >> 16) & 0xFF) as u8,
            b: ((v >> 8) & 0xFF) as u8,
            a: (v & 0xFF) as u8,
        }
    }
}

// ---------------------------------------------------------------------------
// Rect
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    #[inline]
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w as i32 && py < self.y + self.h as i32
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w as i32
            && self.x + self.w as i32 > other.x
            && self.y < other.y + other.h as i32
            && self.y + self.h as i32 > other.y
    }

    pub fn clip(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = (self.x + self.w as i32).min(other.x + other.w as i32);
        let y1 = (self.y + self.h as i32).min(other.y + other.h as i32);
        Some(Rect::new(x0, y0, (x1 - x0) as u32, (y1 - y0) as u32))
    }
}

// ---------------------------------------------------------------------------
// Framebuffer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width as usize) * (height as usize)],
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn raw_data(&self) -> &[u32] {
        &self.pixels
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            self.pixels[(y as usize) * (self.width as usize) + (x as usize)] = color.pack_rgba();
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x < self.width && y < self.height {
            Color::unpack_rgba(self.pixels[(y as usize) * (self.width as usize) + (x as usize)])
        } else {
            Color::TRANSPARENT
        }
    }

    pub fn clear(&mut self, color: Color) {
        let packed = color.pack_rgba();
        self.pixels.fill(packed);
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let fb_rect = Rect::new(0, 0, self.width, self.height);
        let Some(clipped) = rect.clip(&fb_rect) else { return };
        let packed = color.pack_rgba();
        for row in 0..clipped.h {
            let y = clipped.y + row as i32;
            let start = (y as usize) * (self.width as usize) + (clipped.x as usize);
            let end = start + clipped.w as usize;
            self.pixels[start..end].fill(packed);
        }
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        if rect.w == 0 || rect.h == 0 {
            return;
        }
        // top
        self.draw_line(rect.x, rect.y, rect.x + rect.w as i32 - 1, rect.y, color);
        // bottom
        self.draw_line(rect.x, rect.y + rect.h as i32 - 1, rect.x + rect.w as i32 - 1, rect.y + rect.h as i32 - 1, color);
        // left
        self.draw_line(rect.x, rect.y, rect.x, rect.y + rect.h as i32 - 1, color);
        // right
        self.draw_line(rect.x + rect.w as i32 - 1, rect.y, rect.x + rect.w as i32 - 1, rect.y + rect.h as i32 - 1, color);
    }

    /// Bresenham's line algorithm
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx: i32 = if x0 < x1 { 1 } else { -1 };
        let sy: i32 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut cx, mut cy) = (x0, y0);
        loop {
            if cx >= 0 && cy >= 0 {
                self.set_pixel(cx as u32, cy as u32, color);
            }
            if cx == x1 && cy == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                cx += sx;
            }
            if e2 <= dx {
                err += dx;
                cy += sy;
            }
        }
    }

    /// Midpoint circle algorithm — outline
    pub fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        if r < 0 {
            return;
        }
        let mut x = 0i32;
        let mut y = r;
        let mut d = 1 - r;
        while x <= y {
            self.plot_circle_points(cx, cy, x, y, color);
            x += 1;
            if d < 0 {
                d += 2 * x + 1;
            } else {
                y -= 1;
                d += 2 * (x - y) + 1;
            }
        }
    }

    fn plot_circle_points(&mut self, cx: i32, cy: i32, x: i32, y: i32, color: Color) {
        let pts = [
            (cx + x, cy + y),
            (cx - x, cy + y),
            (cx + x, cy - y),
            (cx - x, cy - y),
            (cx + y, cy + x),
            (cx - y, cy + x),
            (cx + y, cy - x),
            (cx - y, cy - x),
        ];
        for &(px, py) in &pts {
            if px >= 0 && py >= 0 {
                self.set_pixel(px as u32, py as u32, color);
            }
        }
    }

    /// Filled circle — midpoint, draw horizontal spans
    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        if r < 0 {
            return;
        }
        let mut x = 0i32;
        let mut y = r;
        let mut d = 1 - r;
        while x <= y {
            self.draw_line(cx - x, cy + y, cx + x, cy + y, color);
            self.draw_line(cx - x, cy - y, cx + x, cy - y, color);
            self.draw_line(cx - y, cy + x, cx + y, cy + x, color);
            self.draw_line(cx - y, cy - x, cx + y, cy - x, color);
            x += 1;
            if d < 0 {
                d += 2 * x + 1;
            } else {
                y -= 1;
                d += 2 * (x - y) + 1;
            }
        }
    }

    /// Alpha-blended blit from src at position (x, y)
    pub fn blit(&mut self, src: &Framebuffer, x: i32, y: i32) {
        let fb_rect = Rect::new(0, 0, self.width, self.height);
        let src_rect = Rect::new(x, y, src.width, src.height);
        let Some(clipped) = src_rect.clip(&fb_rect) else { return };

        for row in 0..clipped.h {
            let dst_y = (clipped.y + row as i32) as usize;
            let src_row = ((clipped.y - y) + row as i32) as usize;
            for col in 0..clipped.w {
                let dst_x = (clipped.x + col as i32) as usize;
                let src_col = ((clipped.x - x) + col as i32) as usize;

                let src_color = Color::unpack_rgba(src.pixels[src_row * src.width as usize + src_col]);
                if src_color.a == 0 {
                    continue;
                }

                let dst_idx = dst_y * self.width as usize + dst_x;

                if src_color.a == 255 {
                    self.pixels[dst_idx] = src_color.pack_rgba();
                } else {
                    let dst_color = Color::unpack_rgba(self.pixels[dst_idx]);
                    let out = alpha_blend(dst_color, src_color);
                    self.pixels[dst_idx] = out.pack_rgba();
                }
            }
        }
    }
}

fn alpha_blend(dst: Color, src: Color) -> Color {
    let sa = src.a as f32 / 255.0;
    let da = dst.a as f32 / 255.0;
    let one_minus_sa = 1.0 - sa;
    let out_a = sa + da * one_minus_sa;
    if out_a == 0.0 {
        return Color::TRANSPARENT;
    }
    Color {
        r: ((src.r as f32 * sa + dst.r as f32 * da * one_minus_sa) / out_a).round() as u8,
        g: ((src.g as f32 * sa + dst.g as f32 * da * one_minus_sa) / out_a).round() as u8,
        b: ((src.b as f32 * sa + dst.b as f32 * da * one_minus_sa) / out_a).round() as u8,
        a: (out_a * 255.0).round() as u8,
    }
}

// ---------------------------------------------------------------------------
// DrawCommand
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrawCommand {
    Clear(Color),
    FillRect(Rect, Color),
    DrawRect(Rect, Color),
    DrawLine(i32, i32, i32, i32, Color),
    DrawCircle(i32, i32, i32, Color),
    FillCircle(i32, i32, i32, Color),
    Blit(u32, i32, i32),
}

// ---------------------------------------------------------------------------
// DrawList
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DrawList {
    pub commands: Vec<DrawCommand>,
    pub textures: HashMap<u32, Framebuffer>,
}

impl Default for DrawList {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawList {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            textures: HashMap::new(),
        }
    }

    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn add_texture(&mut self, id: u32, fb: Framebuffer) {
        self.textures.insert(id, fb);
    }

    pub fn execute(&self, target: &mut Framebuffer) {
        for cmd in &self.commands {
            match cmd {
                DrawCommand::Clear(c) => target.clear(*c),
                DrawCommand::FillRect(r, c) => target.fill_rect(*r, *c),
                DrawCommand::DrawRect(r, c) => target.draw_rect(*r, *c),
                DrawCommand::DrawLine(x0, y0, x1, y1, c) => target.draw_line(*x0, *y0, *x1, *y1, *c),
                DrawCommand::DrawCircle(cx, cy, r, c) => target.draw_circle(*cx, *cy, *r, *c),
                DrawCommand::FillCircle(cx, cy, r, c) => target.fill_circle(*cx, *cy, *r, *c),
                DrawCommand::Blit(id, x, y) => {
                    if let Some(src) = self.textures.get(id) {
                        target.blit(src, *x, *y);
                    }
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

// ---------------------------------------------------------------------------
// TileMap
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TileMap {
    pub tiles: Vec<Vec<u32>>,
    pub tile_size: u32,
    pub tileset: HashMap<u32, Framebuffer>,
}

impl TileMap {
    pub fn new(width: u32, height: u32, tile_size: u32) -> Self {
        Self {
            tiles: vec![vec![0u32; width as usize]; height as usize],
            tile_size,
            tileset: HashMap::new(),
        }
    }

    pub fn set_tile(&mut self, x: u32, y: u32, tile_id: u32) {
        if (y as usize) < self.tiles.len() && (x as usize) < self.tiles[0].len() {
            self.tiles[y as usize][x as usize] = tile_id;
        }
    }

    pub fn get_tile(&self, x: u32, y: u32) -> u32 {
        if (y as usize) < self.tiles.len() && (x as usize) < self.tiles[0].len() {
            self.tiles[y as usize][x as usize]
        } else {
            0
        }
    }

    pub fn render(&self, target: &mut Framebuffer, camera_x: i32, camera_y: i32) {
        let ts = self.tile_size as i32;
        let map_h = self.tiles.len() as i32;
        let map_w = if map_h > 0 { self.tiles[0].len() as i32 } else { 0 };

        // Determine visible tile range
        let start_col = (camera_x / ts).max(0).min(map_w);
        let start_row = (camera_y / ts).max(0).min(map_h);
        let end_col = ((camera_x + target.width as i32 + ts - 1) / ts).min(map_w);
        let end_row = ((camera_y + target.height as i32 + ts - 1) / ts).min(map_h);

        for row in start_row..end_row {
            for col in start_col..end_col {
                let tile_id = self.tiles[row as usize][col as usize];
                if tile_id == 0 {
                    continue;
                }
                if let Some(tile) = self.tileset.get(&tile_id) {
                    let px = col * ts - camera_x;
                    let py = row * ts - camera_y;
                    target.blit(tile, px, py);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Color tests --

    #[test]
    fn color_rgb() {
        let c = Color::rgb(10, 20, 30);
        assert_eq!(c, Color { r: 10, g: 20, b: 30, a: 255 });
    }

    #[test]
    fn color_rgba() {
        let c = Color::rgba(1, 2, 3, 128);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn color_from_hex_6() {
        assert_eq!(Color::from_hex("#ff8040"), Some(Color::rgb(255, 128, 64)));
    }

    #[test]
    fn color_from_hex_8() {
        assert_eq!(Color::from_hex("#ff804080"), Some(Color::rgba(255, 128, 64, 128)));
    }

    #[test]
    fn color_to_hex_roundtrip() {
        let c = Color::rgb(255, 128, 64);
        assert_eq!(Color::from_hex(&c.to_hex()), Some(c));
    }

    #[test]
    fn color_lerp_midpoint() {
        let a = Color::rgb(0, 0, 0);
        let b = Color::rgb(100, 200, 50);
        let mid = a.lerp(&b, 0.5);
        assert_eq!(mid.r, 50);
        assert_eq!(mid.g, 100);
        assert_eq!(mid.b, 25);
    }

    #[test]
    fn color_lerp_zero() {
        let a = Color::rgb(10, 20, 30);
        let b = Color::rgb(99, 99, 99);
        assert_eq!(a.lerp(&b, 0.0), a);
    }

    #[test]
    fn color_premultiply() {
        let c = Color::rgba(200, 100, 50, 128);
        let (r, g, b) = c.premultiply();
        assert_eq!(r, 100);
        assert_eq!(g, 50);
        assert_eq!(b, 25);
    }

    #[test]
    fn color_pack_unpack_roundtrip() {
        let c = Color::rgba(12, 34, 56, 78);
        assert_eq!(Color::unpack_rgba(c.pack_rgba()), c);
    }

    #[test]
    fn color_constants() {
        assert_eq!(Color::WHITE.a, 255);
        assert_eq!(Color::TRANSPARENT.a, 0);
    }

    // -- Rect tests --

    #[test]
    fn rect_contains_inside() {
        let r = Rect::new(5, 5, 10, 10);
        assert!(r.contains(7, 7));
    }

    #[test]
    fn rect_contains_edge() {
        let r = Rect::new(0, 0, 5, 5);
        assert!(r.contains(0, 0));
        assert!(!r.contains(5, 5)); // exclusive
    }

    #[test]
    fn rect_intersects() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        assert!(a.intersects(&b));
    }

    #[test]
    fn rect_no_intersect() {
        let a = Rect::new(0, 0, 5, 5);
        let b = Rect::new(10, 10, 5, 5);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn rect_clip() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        let c = a.clip(&b).unwrap();
        assert_eq!(c, Rect::new(5, 5, 5, 5));
    }

    // -- Framebuffer tests --

    #[test]
    fn fb_new_transparent() {
        let fb = Framebuffer::new(10, 10);
        assert_eq!(fb.get_pixel(0, 0), Color::TRANSPARENT);
    }

    #[test]
    fn fb_set_get_pixel() {
        let mut fb = Framebuffer::new(10, 10);
        fb.set_pixel(3, 4, Color::RED);
        assert_eq!(fb.get_pixel(3, 4), Color::RED);
    }

    #[test]
    fn fb_set_pixel_out_of_bounds() {
        let mut fb = Framebuffer::new(5, 5);
        fb.set_pixel(10, 10, Color::RED); // should not panic
    }

    #[test]
    fn fb_clear() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::GREEN);
        assert_eq!(fb.get_pixel(9, 9), Color::GREEN);
    }

    #[test]
    fn fb_fill_rect() {
        let mut fb = Framebuffer::new(20, 20);
        fb.fill_rect(Rect::new(5, 5, 5, 5), Color::BLUE);
        assert_eq!(fb.get_pixel(7, 7), Color::BLUE);
        assert_eq!(fb.get_pixel(4, 7), Color::TRANSPARENT);
        assert_eq!(fb.get_pixel(10, 7), Color::TRANSPARENT);
    }

    #[test]
    fn fb_draw_rect_outline() {
        let mut fb = Framebuffer::new(10, 10);
        fb.draw_rect(Rect::new(2, 2, 4, 4), Color::WHITE);
        // corners
        assert_eq!(fb.get_pixel(2, 2), Color::WHITE);
        assert_eq!(fb.get_pixel(5, 2), Color::WHITE);
        // inside should be transparent
        assert_eq!(fb.get_pixel(3, 3), Color::TRANSPARENT);
    }

    #[test]
    fn fb_draw_line_horizontal() {
        let mut fb = Framebuffer::new(20, 20);
        fb.draw_line(0, 5, 9, 5, Color::RED);
        for x in 0..=9 {
            assert_eq!(fb.get_pixel(x, 5), Color::RED);
        }
    }

    #[test]
    fn fb_draw_line_diagonal() {
        let mut fb = Framebuffer::new(10, 10);
        fb.draw_line(0, 0, 4, 4, Color::WHITE);
        for i in 0..=4 {
            assert_eq!(fb.get_pixel(i, i), Color::WHITE);
        }
    }

    #[test]
    fn fb_draw_circle() {
        let mut fb = Framebuffer::new(20, 20);
        fb.draw_circle(10, 10, 5, Color::WHITE);
        // cardinal points at radius 5
        assert_eq!(fb.get_pixel(15, 10), Color::WHITE);
        assert_eq!(fb.get_pixel(5, 10), Color::WHITE);
        assert_eq!(fb.get_pixel(10, 15), Color::WHITE);
        assert_eq!(fb.get_pixel(10, 5), Color::WHITE);
        // center should be untouched
        assert_eq!(fb.get_pixel(10, 10), Color::TRANSPARENT);
    }

    #[test]
    fn fb_fill_circle() {
        let mut fb = Framebuffer::new(20, 20);
        fb.fill_circle(10, 10, 3, Color::GREEN);
        assert_eq!(fb.get_pixel(10, 10), Color::GREEN);
        // outside
        assert_eq!(fb.get_pixel(0, 0), Color::TRANSPARENT);
    }

    #[test]
    fn fb_blit_full_opaque() {
        let mut dst = Framebuffer::new(10, 10);
        let mut src = Framebuffer::new(4, 4);
        src.clear(Color::RED);
        dst.blit(&src, 3, 3);
        assert_eq!(dst.get_pixel(3, 3), Color::RED);
        assert_eq!(dst.get_pixel(6, 6), Color::RED);
        assert_eq!(dst.get_pixel(2, 3), Color::TRANSPARENT);
    }

    #[test]
    fn fb_blit_alpha_blend() {
        let mut dst = Framebuffer::new(10, 10);
        dst.clear(Color::WHITE);
        let mut src = Framebuffer::new(5, 5);
        src.clear(Color::rgba(0, 0, 0, 128));
        dst.blit(&src, 0, 0);
        let p = dst.get_pixel(2, 2);
        // semi-transparent black over white → mid gray
        assert!(p.r > 100 && p.r < 200, "expected mid gray, got {}", p.r);
    }

    #[test]
    fn fb_blit_clipped() {
        let mut dst = Framebuffer::new(5, 5);
        let mut src = Framebuffer::new(10, 10);
        src.clear(Color::RED);
        dst.blit(&src, -3, -3);
        // src at (-3,-3), visible region is entire 5x5 dst, all red
        assert_eq!(dst.get_pixel(0, 0), Color::RED);
        assert_eq!(dst.get_pixel(4, 4), Color::RED);
    }

    // -- DrawList tests --

    #[test]
    fn drawlist_execute_clear() {
        let mut dl = DrawList::new();
        dl.push(DrawCommand::Clear(Color::RED));
        let mut fb = Framebuffer::new(10, 10);
        dl.execute(&mut fb);
        assert_eq!(fb.get_pixel(5, 5), Color::RED);
    }

    #[test]
    fn drawlist_execute_multiple() {
        let mut dl = DrawList::new();
        dl.push(DrawCommand::Clear(Color::BLACK));
        dl.push(DrawCommand::FillRect(Rect::new(0, 0, 5, 5), Color::RED));
        assert_eq!(dl.len(), 2);
        let mut fb = Framebuffer::new(10, 10);
        dl.execute(&mut fb);
        assert_eq!(fb.get_pixel(0, 0), Color::RED);
        assert_eq!(fb.get_pixel(5, 5), Color::BLACK);
    }

    #[test]
    fn drawlist_blit_texture() {
        let mut dl = DrawList::new();
        let mut tex = Framebuffer::new(3, 3);
        tex.clear(Color::GREEN);
        dl.add_texture(1, tex);
        dl.push(DrawCommand::Clear(Color::BLACK));
        dl.push(DrawCommand::Blit(1, 2, 2));
        let mut fb = Framebuffer::new(10, 10);
        dl.execute(&mut fb);
        assert_eq!(fb.get_pixel(3, 3), Color::GREEN);
    }

    #[test]
    fn drawlist_clear_commands() {
        let mut dl = DrawList::new();
        dl.push(DrawCommand::Clear(Color::WHITE));
        dl.clear();
        assert_eq!(dl.len(), 0);
    }

    // -- TileMap tests --

    #[test]
    fn tilemap_set_get() {
        let mut tm = TileMap::new(5, 5, 16);
        tm.set_tile(2, 3, 42);
        assert_eq!(tm.get_tile(2, 3), 42);
        assert_eq!(tm.get_tile(0, 0), 0);
    }

    #[test]
    fn tilemap_render() {
        let mut tm = TileMap::new(3, 3, 4);
        let mut tile = Framebuffer::new(4, 4);
        tile.clear(Color::BLUE);
        tm.tileset.insert(1, tile);
        tm.set_tile(0, 0, 1);
        tm.set_tile(1, 0, 1);
        let mut fb = Framebuffer::new(8, 4);
        tm.render(&mut fb, 0, 0);
        // tile (0,0) fills 0..4, tile (1,0) fills 4..8
        assert_eq!(fb.get_pixel(0, 0), Color::BLUE);
        assert_eq!(fb.get_pixel(4, 0), Color::BLUE);
        // tile (2,0) is 0, so should be transparent
        // but fb is 8 wide, column 8 doesn't exist — all good
    }

    #[test]
    fn tilemap_render_with_camera() {
        let mut tm = TileMap::new(3, 3, 8);
        let mut tile = Framebuffer::new(8, 8);
        tile.clear(Color::RED);
        tm.tileset.insert(1, tile);
        tm.set_tile(0, 0, 1);
        let mut fb = Framebuffer::new(8, 8);
        tm.render(&mut fb, 4, 4); // camera offset
        // tile (0,0) starts at pixel (-4,-4), so only bottom-right quadrant visible
        assert_eq!(fb.get_pixel(0, 0), Color::RED);
        assert_eq!(fb.get_pixel(3, 3), Color::RED);
    }

    // -- Determinism test --

    #[test]
    fn determinism_same_commands_same_pixels() {
        let mut dl = DrawList::new();
        dl.push(DrawCommand::Clear(Color::BLACK));
        dl.push(DrawCommand::FillRect(Rect::new(10, 10, 30, 30), Color::RED));
        dl.push(DrawCommand::DrawCircle(25, 25, 10, Color::WHITE));
        dl.push(DrawCommand::DrawLine(0, 0, 49, 49, Color::GREEN));

        let mut fb1 = Framebuffer::new(50, 50);
        dl.execute(&mut fb1);

        let mut fb2 = Framebuffer::new(50, 50);
        dl.execute(&mut fb2);

        assert_eq!(fb1.raw_data(), fb2.raw_data());
    }

    // -- Serde tests --

    #[test]
    fn serde_color_roundtrip() {
        let c = Color::rgba(1, 2, 3, 4);
        let json = serde_json::to_string(&c).unwrap();
        let c2: Color = serde_json::from_str(&json).unwrap();
        assert_eq!(c, c2);
    }

    #[test]
    fn serde_rect_roundtrip() {
        let r = Rect::new(10, 20, 30, 40);
        let json = serde_json::to_string(&r).unwrap();
        let r2: Rect = serde_json::from_str(&json).unwrap();
        assert_eq!(r, r2);
    }

    #[test]
    fn serde_draw_command() {
        let cmd = DrawCommand::DrawLine(1, 2, 3, 4, Color::RED);
        let json = serde_json::to_string(&cmd).unwrap();
        let cmd2: DrawCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(json, serde_json::to_string(&cmd2).unwrap());
    }

    #[test]
    fn fb_raw_data_len() {
        let fb = Framebuffer::new(16, 9);
        assert_eq!(fb.raw_data().len(), 16 * 9);
    }

    #[test]
    fn fb_dimensions() {
        let fb = Framebuffer::new(32, 24);
        assert_eq!(fb.width(), 32);
        assert_eq!(fb.height(), 24);
    }
}
