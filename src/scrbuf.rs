//! Screen bufer - collects what needs to be painted and paints it using the palette.

use crate::{GfxData, Painter, RGB};

// Special scaler, for correctly rendering walls and sprites in 3D view
const PIC_HEIGHT_SCALER: f64 = 1.1;
const ADJUST_EPSILON: f64 = 0.125;

/// Screen buffer - holds one buffer of screen data and paints it on the screen.
pub struct ScreenBuffer {
    width: i32,
    height: i32,
    screen_x_start: i32,
    screen_y_start: i32,
    view_height: i32,
    bytes: Vec<u8>,
    use_sod_palette: bool,
    dist_from_screen: f64,
    hfov: f64,
    wall_heights: Vec<i32>,
}

impl ScreenBuffer {
    /// Create a new screen buffer.
    pub fn new(scr_width: i32, scr_height: i32, use_sod_palette: bool) -> Self {
        // adjust width and height, so that it is 4/3
        let scale = if (scr_width * 3) > (scr_height * 4) {
            // too wide => use height as basis
            scr_height / 12
        } else {
            // (maybe) height too large => use width as basis
            scr_width / 16
        };
        let height = scale * 12;
        let width = scale * 16;
        let screen_x_start = (scr_width - width) / 2;
        let screen_y_start = (scr_height - height) / 2;

        let len = (width * height) as usize;
        let (dist_from_screen, hfov) = compute_dist_from_screen_and_hfov(width, height);
        Self {
            width,
            height,
            screen_x_start,
            screen_y_start,
            view_height: height,
            bytes: vec![0; len],
            use_sod_palette,
            dist_from_screen,
            hfov,
            wall_heights: vec![0; width as usize],
        }
    }

    /// Screen buffer width.
    #[inline]
    pub fn scr_width(&self) -> i32 {
        self.width
    }

    /// Screen buffer height.
    #[inline]
    pub fn scr_height(&self) -> i32 {
        self.height
    }

    /// 3D view height (without statusbar reserved space).
    #[inline]
    pub fn view_height(&self) -> i32 {
        self.view_height
    }

    /// Enable/disable status bar reserved space.
    #[inline]
    pub fn enable_status_bar(&mut self, enabled: bool) {
        self.view_height = if enabled { self.height * 4 / 5 } else { self.height };
    }

    /// Put a pixel in the buffer, *with* transparency.
    #[inline]
    pub fn put_pixel(&mut self, x: i32, y: i32, c: u8) {
        if x >= 0 && y >= 0 && c != 0xFF && x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.bytes[idx as usize] = c;
        }
    }

    /// Fill a rectangle inside the buffer, *without* transparency.
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        if w <= 0 || h <= 0 || x >= self.width || y >= self.height {
            return;
        }

        // shift top-left corner inside the screen
        let xx = Ord::max(x, 0);
        let yy = Ord::max(y, 0);
        let w = w - xx + x;
        let h = h - yy + y;
        if w <= 0 || h <= 0 {
            return;
        }

        // shift bottom right corner inside the screen
        let sw = Ord::min(w, self.width - xx);
        let sh = Ord::min(h, self.height - yy);
        let mut idx = (yy * self.width + xx) as usize;
        let step = self.width - sw;

        // ok to paint
        for _ in 0..sh {
            for _ in 0..sw {
                self.bytes[idx] = c;
                idx += 1;
            }
            idx += step as usize;
        }
    }

    /*
    /// Rather inefficient way to draw a line.
    /// (Does not matter, it is only used in a few places)
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: u8) {
        let mut x = x1 as f64;
        let mut y = y1 as f64;
        let dist_x = x2 - x1 + 1;
        let dist_y = y2 - y1 + 1;
        let (cnt_pixels, dx, dy) = if dist_x.abs() >= dist_y.abs() {
            let dist = dist_x.abs();
            let dy = ((y2 - y1 + 1) as f64) / (dist as f64);
            (dist, dist_x.signum() as f64, dy)
        } else {
            let dist = dist_y.abs();
            let dx = ((x2 - x1 + 1) as f64) / (dist as f64);
            (dist, dx, dist_y.signum() as f64)
        };
        for _ in 0..cnt_pixels {
            self.put_pixel(x as i32, y as i32, c);
            x += dx;
            y += dy;
        }
    } */

    /// Init 3D view - paint sky and floor.
    pub fn clear_3d_view(&mut self, sky_color: u8) {
        const FLOOR_COLOR: u8 = 0x19;
        let halfh = self.view_height >> 1;
        self.fill_rect(0, 0, self.width, halfh, sky_color);
        self.fill_rect(0, halfh, self.width, halfh, FLOOR_COLOR);
    }

    /// Render one column of a texture, centered vertically and proportionally scaled, in 3D mode.
    /// * `screen_x` = x position on the screem where to paint.
    /// * `dist` = distance to the wall/sprite, in map space
    /// * `tex_x_rel_ofs` = relative offset within the texture (0.0 = left-most edge, 1.0 = right-most edge).
    /// * `texture` = the source texture, for texturing the rendered column of pixels.
    pub fn render_texture_column(&mut self, screen_x: i32, dist: f64, tex_x_rel_ofs: f64, texture: &GfxData) {
        if screen_x < 0 || screen_x >= self.width || dist < 0.004 {
            // the column is outside the screen OR too near => no need to paint it :)
            return;
        }

        // not very optimal, but it works...
        let height_scale = PIC_HEIGHT_SCALER / dist;

        // adjust with an epsilon, to avoid errors in the texture
        // (usually missing pixels on the edge of a wall/door)
        let scaled_height = ((self.height as f64) * height_scale + ADJUST_EPSILON) as i32;
        self.wall_heights[screen_x as usize] = scaled_height;

        let dystep = 1.0 / (scaled_height as f64);
        let mut dy = 0.0;
        let mut y = (self.view_height - scaled_height) / 2;
        for _ in 0..scaled_height {
            if y >= 0 && y < self.view_height {
                let texel = texture.texel(tex_x_rel_ofs, dy);
                self.put_pixel(screen_x, y, texel);
            }
            y += 1;
            dy += dystep;
        }
    }

    pub fn render_sprite(&mut self, angle: f64, dist: f64, sprite: &GfxData) {
        if dist < 0.004 {
            // the sprite is too near => no need to paint it :)
            return;
        }

        // TODO tune this - seems a bit off, compared to ECWolf
        let half_sprite_view_angle = (0.5 / dist).atan();
        let x1 = self.angle_to_screen_x(angle - half_sprite_view_angle);
        let x2 = self.angle_to_screen_x(angle + half_sprite_view_angle);

        let height_scale = PIC_HEIGHT_SCALER / dist;
        let scaled_height = ((self.height as f64) * height_scale + ADJUST_EPSILON) as i32;

        let tex_step = 1.0 / ((x2 - x1 + 1) as f64);
        let mut tex_x = 0.0;
        for x in x1..=x2 {
            if x >= 0 && x < self.width {
                let wallh = self.wall_heights[x as usize];
                if scaled_height >= wallh {
                    self.render_texture_column(x, dist, tex_x, sprite);
                    self.wall_heights[x as usize] = wallh;
                }
            }
            tex_x += tex_step;
        }
    }

    /// Draw a picture proportionally scaled, in 2D mode.
    pub fn draw_scaled_pic(&mut self, x: i32, y: i32, scaled_width: i32, scaled_height: i32, sprite: &GfxData) {
        let spr_size = sprite.size();
        if spr_size.0 <= 0 || spr_size.1 <= 0 {
            // Trying to paint empty sprite
            return;
        }
        if scaled_width < 2 || scaled_height < 2 {
            // Trying to paint sprite too small
            return;
        }

        let x_step = 1.0 / (scaled_width as f64);
        let y_step = 1.0 / (scaled_height as f64);

        let mut dx = 0.0;
        for scr_x in x..x + scaled_width {
            if scr_x >= 0 && scr_x < self.width {
                let mut dy = 0.0;
                for scr_y in y..y + scaled_height {
                    self.put_pixel(scr_x, scr_y, sprite.texel(dx, dy));
                    dy += y_step;
                }
            }
            dx += x_step;
        }
    }

    pub fn draw_player_weapon_sprite(&mut self, weapon_sprite: &GfxData) {
        let scaled_height = self.height * 4 / 5;
        let scaled_width = self.height * 2 / 3;
        let xo = (self.width - scaled_width) / 2;
        let yo = self.view_height - scaled_height;
        self.draw_scaled_pic(xo, yo, scaled_width, scaled_height, weapon_sprite);
    }

    /// Paint the buffer onto the screen.
    pub fn paint(&self, painter: &mut dyn Painter) {
        let mut idx = 0;
        for y in 0..(self.height as i32) {
            for x in 0..(self.width as i32) {
                let color = palette_to_rgb(self.bytes[idx], self.use_sod_palette);
                painter.draw_pixel(x + self.screen_x_start, y + self.screen_y_start, color);
                idx += 1;
            }
        }
    }

    #[inline]
    pub fn half_fov(&self) -> f64 {
        self.hfov
    }

    #[inline]
    pub fn screen_x_to_angle(&self, screen_x: i32) -> f64 {
        // I just mirrored the screen horizontally, because map layout is y-flipped :(
        let dx_from_screen_center = (screen_x - self.width / 2) as f64;
        dx_from_screen_center.atan2(self.dist_from_screen)
    }

    #[inline]
    pub fn angle_to_screen_x(&self, angle: f64) -> i32 {
        let dx_from_screen_center = (angle.tan() * self.dist_from_screen) as i32;
        dx_from_screen_center + (self.width / 2)
    }
}

// NOTE: the palettes of Wolf3D and SOD are different for only 2 colors:
//      166 => RGB(0, 56, 0)
//      167 => RGB(0, 40, 0)
#[inline]
pub fn palette_to_rgb(c: u8, sod: bool) -> RGB {
    if sod {
        match c {
            166 => {
                return RGB::from(0, 56, 0);
            }
            167 => {
                return RGB::from(0, 40, 0);
            }
            _ => {}
        }
    }
    let idx = (c as usize) * 3;
    RGB::from(PALETTE[idx], PALETTE[idx + 1], PALETTE[idx + 2])
}

//--------------------------
//  Internal stuff

/// Computes the "virtual" distance from the screen (in "pixels") and half-FOV.
/// Assumes a 4/3 screen ratio and a full FOV of LESS THAN 90 degrees.
/// -> for 90 degrees, the 1st expression should have: ... * 2.0 / 3.0 !!
fn compute_dist_from_screen_and_hfov(width: i32, height: i32) -> (f64, f64) {
    let dist_from_screen = (height as f64) * 2.75 / 3.0;
    assert!(dist_from_screen > 1.0);
    let half_width = (width as f64) / 2.0;
    let hfov = half_width.atan2(dist_from_screen);
    (dist_from_screen, hfov)
}

const PALETTE: &[u8] = &[
    0x00, 0x00, 0x00, 0x00, 0x00, 0xA8, 0x00, 0xA8, 0x00, 0x00, 0xA8, 0xA8, 0xA8, 0x00, 0x00, 0xA8, 0x00, 0xA8, 0xA8,
    0x54, 0x00, 0xA8, 0xA8, 0xA8, 0x54, 0x54, 0x54, 0x54, 0x54, 0xFF, 0x54, 0xFF, 0x54, 0x54, 0xFF, 0xFF, 0xFF, 0x54,
    0x54, 0xFF, 0x54, 0xFF, 0xFF, 0xFF, 0x54, 0xFF, 0xFF, 0xFF, 0xEC, 0xEC, 0xEC, 0xDC, 0xDC, 0xDC, 0xD0, 0xD0, 0xD0,
    0xC0, 0xC0, 0xC0, 0xB4, 0xB4, 0xB4, 0xA8, 0xA8, 0xA8, 0x98, 0x98, 0x98, 0x8C, 0x8C, 0x8C, 0x7C, 0x7C, 0x7C, 0x70,
    0x70, 0x70, 0x64, 0x64, 0x64, 0x54, 0x54, 0x54, 0x48, 0x48, 0x48, 0x38, 0x38, 0x38, 0x2C, 0x2C, 0x2C, 0x20, 0x20,
    0x20, 0xFF, 0x00, 0x00, 0xEC, 0x00, 0x00, 0xE0, 0x00, 0x00, 0xD4, 0x00, 0x00, 0xC8, 0x00, 0x00, 0xBC, 0x00, 0x00,
    0xB0, 0x00, 0x00, 0xA4, 0x00, 0x00, 0x98, 0x00, 0x00, 0x88, 0x00, 0x00, 0x7C, 0x00, 0x00, 0x70, 0x00, 0x00, 0x64,
    0x00, 0x00, 0x58, 0x00, 0x00, 0x4C, 0x00, 0x00, 0x40, 0x00, 0x00, 0xFF, 0xD8, 0xD8, 0xFF, 0xB8, 0xB8, 0xFF, 0x9C,
    0x9C, 0xFF, 0x7C, 0x7C, 0xFF, 0x5C, 0x5C, 0xFF, 0x40, 0x40, 0xFF, 0x20, 0x20, 0xFF, 0x00, 0x00, 0xFF, 0xA8, 0x5C,
    0xFF, 0x98, 0x40, 0xFF, 0x88, 0x20, 0xFF, 0x78, 0x00, 0xE4, 0x6C, 0x00, 0xCC, 0x60, 0x00, 0xB4, 0x54, 0x00, 0x9C,
    0x4C, 0x00, 0xFF, 0xFF, 0xD8, 0xFF, 0xFF, 0xB8, 0xFF, 0xFF, 0x9C, 0xFF, 0xFF, 0x7C, 0xFF, 0xF8, 0x5C, 0xFF, 0xF4,
    0x40, 0xFF, 0xF4, 0x20, 0xFF, 0xF4, 0x00, 0xE4, 0xD8, 0x00, 0xCC, 0xC4, 0x00, 0xB4, 0xAC, 0x00, 0x9C, 0x9C, 0x00,
    0x84, 0x84, 0x00, 0x70, 0x6C, 0x00, 0x58, 0x54, 0x00, 0x40, 0x40, 0x00, 0xD0, 0xFF, 0x5C, 0xC4, 0xFF, 0x40, 0xB4,
    0xFF, 0x20, 0xA0, 0xFF, 0x00, 0x90, 0xE4, 0x00, 0x80, 0xCC, 0x00, 0x74, 0xB4, 0x00, 0x60, 0x9C, 0x00, 0xD8, 0xFF,
    0xD8, 0xBC, 0xFF, 0xB8, 0x9C, 0xFF, 0x9C, 0x80, 0xFF, 0x7C, 0x60, 0xFF, 0x5C, 0x40, 0xFF, 0x40, 0x20, 0xFF, 0x20,
    0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0xEC, 0x00, 0x00, 0xE0, 0x00, 0x00, 0xD4, 0x00, 0x04, 0xC8, 0x00, 0x04,
    0xBC, 0x00, 0x04, 0xB0, 0x00, 0x04, 0xA4, 0x00, 0x04, 0x98, 0x00, 0x04, 0x88, 0x00, 0x04, 0x7C, 0x00, 0x04, 0x70,
    0x00, 0x04, 0x64, 0x00, 0x04, 0x58, 0x00, 0x04, 0x4C, 0x00, 0x04, 0x40, 0x00, 0xD8, 0xFF, 0xFF, 0xB8, 0xFF, 0xFF,
    0x9C, 0xFF, 0xFF, 0x7C, 0xFF, 0xF8, 0x5C, 0xFF, 0xFF, 0x40, 0xFF, 0xFF, 0x20, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0x00,
    0xE4, 0xE4, 0x00, 0xCC, 0xCC, 0x00, 0xB4, 0xB4, 0x00, 0x9C, 0x9C, 0x00, 0x84, 0x84, 0x00, 0x70, 0x70, 0x00, 0x58,
    0x58, 0x00, 0x40, 0x40, 0x5C, 0xBC, 0xFF, 0x40, 0xB0, 0xFF, 0x20, 0xA8, 0xFF, 0x00, 0x9C, 0xFF, 0x00, 0x8C, 0xE4,
    0x00, 0x7C, 0xCC, 0x00, 0x6C, 0xB4, 0x00, 0x5C, 0x9C, 0xD8, 0xD8, 0xFF, 0xB8, 0xBC, 0xFF, 0x9C, 0x9C, 0xFF, 0x7C,
    0x80, 0xFF, 0x5C, 0x60, 0xFF, 0x40, 0x40, 0xFF, 0x20, 0x24, 0xFF, 0x00, 0x04, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00,
    0xEC, 0x00, 0x00, 0xE0, 0x00, 0x00, 0xD4, 0x00, 0x00, 0xC8, 0x00, 0x00, 0xBC, 0x00, 0x00, 0xB0, 0x00, 0x00, 0xA4,
    0x00, 0x00, 0x98, 0x00, 0x00, 0x88, 0x00, 0x00, 0x7C, 0x00, 0x00, 0x70, 0x00, 0x00, 0x64, 0x00, 0x00, 0x58, 0x00,
    0x00, 0x4C, 0x00, 0x00, 0x40, 0x28, 0x28, 0x28, 0xFF, 0xE0, 0x34, 0xFF, 0xD4, 0x24, 0xFF, 0xCC, 0x18, 0xFF, 0xC0,
    0x08, 0xFF, 0xB4, 0x00, 0xB4, 0x20, 0xFF, 0xA8, 0x00, 0xFF, 0x98, 0x00, 0xE4, 0x80, 0x00, 0xCC, 0x74, 0x00, 0xB4,
    0x60, 0x00, 0x9C, 0x50, 0x00, 0x84, 0x44, 0x00, 0x70, 0x34, 0x00, 0x58, 0x28, 0x00, 0x40, 0xFF, 0xD8, 0xFF, 0xFF,
    0xB8, 0xFF, 0xFF, 0x9C, 0xFF, 0xFF, 0x7C, 0xFF, 0xFF, 0x5C, 0xFF, 0xFF, 0x40, 0xFF, 0xFF, 0x20, 0xFF, 0xFF, 0x00,
    0xFF, 0xE0, 0x00, 0xE4, 0xC8, 0x00, 0xCC, 0xB4, 0x00, 0xB4, 0x9C, 0x00, 0x9C, 0x84, 0x00, 0x84, 0x6C, 0x00, 0x70,
    0x58, 0x00, 0x58, 0x40, 0x00, 0x40, 0xFF, 0xE8, 0xDC, 0xFF, 0xE0, 0xD0, 0xFF, 0xD8, 0xC4, 0xFF, 0xD4, 0xBC, 0xFF,
    0xCC, 0xB0, 0xFF, 0xC4, 0xA4, 0xFF, 0xBC, 0x9C, 0xFF, 0xB8, 0x90, 0xFF, 0xB0, 0x80, 0xFF, 0xA4, 0x70, 0xFF, 0x9C,
    0x60, 0xF0, 0x94, 0x5C, 0xE8, 0x8C, 0x58, 0xDC, 0x88, 0x54, 0xD0, 0x80, 0x50, 0xC8, 0x7C, 0x4C, 0xBC, 0x78, 0x48,
    0xB4, 0x70, 0x44, 0xA8, 0x68, 0x40, 0xA0, 0x64, 0x3C, 0x9C, 0x60, 0x38, 0x90, 0x5C, 0x34, 0x88, 0x58, 0x30, 0x80,
    0x50, 0x2C, 0x74, 0x4C, 0x28, 0x6C, 0x48, 0x24, 0x5C, 0x40, 0x20, 0x54, 0x3C, 0x1C, 0x48, 0x38, 0x18, 0x40, 0x30,
    0x18, 0x38, 0x2C, 0x14, 0x28, 0x20, 0x0C, 0x60, 0x00, 0x64, 0x00, 0x64, 0x64, 0x00, 0x60, 0x60, 0x00, 0x00, 0x1C,
    0x00, 0x00, 0x2C, 0x30, 0x24, 0x10, 0x48, 0x00, 0x48, 0x50, 0x00, 0x50, 0x00, 0x00, 0x34, 0x1C, 0x1C, 0x1C, 0x4C,
    0x4C, 0x4C, 0x5C, 0x5C, 0x5C, 0x40, 0x40, 0x40, 0x30, 0x30, 0x30, 0x34, 0x34, 0x34, 0xD8, 0xF4, 0xF4, 0xB8, 0xE8,
    0xE8, 0x9C, 0xDC, 0xDC, 0x74, 0xC8, 0xC8, 0x48, 0xC0, 0xC0, 0x20, 0xB4, 0xB4, 0x20, 0xB0, 0xB0, 0x00, 0xA4, 0xA4,
    0x00, 0x98, 0x98, 0x00, 0x8C, 0x8C, 0x00, 0x84, 0x84, 0x00, 0x7C, 0x7C, 0x00, 0x78, 0x78, 0x00, 0x74, 0x74, 0x00,
    0x70, 0x70, 0x00, 0x6C, 0x6C, 0xFF, 0x00, 0xFF,
];
