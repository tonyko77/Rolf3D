//! Data structures for the assets loaded from files

// TODO how to interpret the map data?
//  -> looks like each map has 64*64 = 4096 words, between 0x00 and 0xFF
//      => it's JUST a 2D array :))
//  -> plane #1 seems to contain WALLS, plane #2 seems to contain THINGS
//  -> plane #3 seems to ALWAYS have 0-s => NOT USED ?!?, check SOD, WL6 etc

//-----------------------

use crate::ScreenBuffer;

/// Graphics - contains walls, sprites and miscellaneous (fonts, PICs etc)
/// Each pic is stored as columns, then rows (flipped)
pub struct GfxData {
    width: u16,
    height: u16,
    texels: Vec<u8>,
}

impl GfxData {
    #[inline]
    pub fn new_sprite(pixels: Vec<u8>) -> Self {
        Self::new_pic(64, 64, pixels)
    }

    #[inline]
    pub fn new_pic(width: u16, height: u16, texels: Vec<u8>) -> Self {
        assert_eq!((width * height) as usize, texels.len());
        Self { width, height, texels }
    }

    #[inline]
    pub fn new_empty() -> Self {
        Self {
            width: 0,
            height: 0,
            texels: vec![],
        }
    }

    #[inline]
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Draw the picture as-is, in 2D mode.
    pub fn draw(&self, x: i32, y: i32, scrbuf: &mut ScreenBuffer) {
        let w = self.width as i32;
        let h = self.height as i32;
        if w == 0 || h == 0 {
            return;
        }

        let mut idx = 0;
        for dx in 0..w {
            for dy in 0..h {
                scrbuf.put_pixel(x + dx, y + dy, self.texels[idx]);
                idx += 1;
            }
        }
    }

    /// Draw the picture proportionally scaled, in 2D mode.
    pub fn draw_scaled(&self, x: i32, y: i32, scaled_width: i32, scrbuf: &mut ScreenBuffer) {
        let w = self.width as i32;
        let h = self.height as i32;
        if w == 0 || h == 0 {
            return;
        }

        let scaled_height = h * scaled_width / w;
        for dx in 0..scaled_width {
            for dy in 0..scaled_height {
                let ddx = Ord::min(dx * w / scaled_width, w - 1);
                let ddy = Ord::min(dy * w / scaled_width, h - 1);
                let idx = (ddx * h + ddy) as usize;
                scrbuf.put_pixel(x + dx, y + dy, self.texels[idx]);
            }
        }
    }

    /// Render one column of the picture, centered vertically and proportionally scaled, in 3D mode.
    /// The height scale is in 1/1000-s of the entire screen height (1000 = full height, 500 = half screen height etc).
    pub fn render_column(&self, src_pic_x: i32, screen_x: i32, height_scale: i32, scrbuf: &mut ScreenBuffer) {
        assert!(self.width > 0 && self.height > 0, "Rendering missing texture");
        assert!(
            height_scale >= 10 && height_scale <= 10000,
            "Extreme height scale when rendering texture"
        );

        if screen_x < 0 || screen_x >= scrbuf.scr_width() {
            // the column is outside the screen => no need to paint it :)
            return;
        }

        let vertc = scrbuf.get_vert_center();
        let scrh = vertc * 2;
        let scaled_height = scrh * height_scale / 1000;

        let srcx = src_pic_x.clamp(0, (self.width - 1) as i32);
        let mut fidx = (srcx * (self.height as i32)) as f64;
        let fstep = (self.height as f64) / (scaled_height as f64);
        let mut y = vertc - (scaled_height / 2);

        for _ in 0..scaled_height {
            if y >= 0 && y < scrh {
                let texel = self.texels[fidx as usize];
                scrbuf.put_pixel(screen_x, y, texel);
            }
            y += 1;
            fidx += fstep;
        }
    }
}

//-----------------------

pub struct FontData {
    font_height: u16,
    space_width: u16,
    offs_widths: Vec<u16>,
    pixels: Vec<u8>,
}

impl FontData {
    pub fn new(font_height: u16, space_width: u16, offs_widths: Vec<u16>, pixels: Vec<u8>) -> Self {
        Self {
            font_height,
            space_width,
            offs_widths,
            pixels,
        }
    }

    pub fn text_width(&self, text: &str) -> i32 {
        let mut dx = 0;
        for ch in text.bytes() {
            let cw = match ch {
                32 => self.space_width as i32,
                33..=127 => {
                    let idx = ((ch - 33) as usize) * 2 + 1;
                    self.offs_widths[idx] as i32
                }
                _ => 0,
            };
            dx += cw;
        }
        dx
    }

    pub fn draw_text(&self, x: i32, y: i32, text: &str, color: u8, scrbuf: &mut ScreenBuffer) -> i32 {
        let mut dx = 0;
        for ch in text.bytes() {
            let cw = self.draw_char(x + dx, y, ch, color, scrbuf);
            dx += cw;
        }
        dx
    }

    pub fn draw_char(&self, x: i32, y: i32, ch: u8, color: u8, scrbuf: &mut ScreenBuffer) -> i32 {
        // if not a drawable char => just skip it
        if ch < 32 || ch > 127 {
            return 0;
        }
        // if a space => draw nothing, just return its width
        if ch == 32 {
            return self.space_width as i32;
        }
        // ok to draw
        let idx = ((ch - 33) as usize) * 2;
        let mut ofs = self.offs_widths[idx] as usize; // first word = offset inside pixels
        let width = self.offs_widths[idx + 1] as i32; // second word = width of character
        let height = self.font_height as i32;
        for dx in 0..width {
            for dy in 0..height {
                if 0 != self.pixels[ofs] {
                    scrbuf.put_pixel(x + dx, y + dy, color);
                }
                ofs += 1;
            }
        }

        width
    }
}

//-----------------------

/// Map data - contains walls/doors and things.
/// Note: all levels have a size of 64x64, but we keep the width and height
/// as explicit values here, for flexibility.
pub struct MapData {
    pub name: String,
    pub width: u16,
    pub height: u16,
    tiles: Vec<u16>,
    things: Vec<u16>,
}

impl MapData {
    pub fn new(name: String, width: u16, height: u16, tiles: Vec<u16>, things: Vec<u16>) -> Self {
        // some silly checks - seem to be valid for WL1, WL6 and SOD
        assert!(name.len() > 0);
        // check that maps are always 64 x 64
        assert_eq!(64, width);
        assert_eq!(64, height);
        // check that planes have exactly 64*64 = 4096 words
        assert_eq!(4096, tiles.len());
        assert_eq!(4096, things.len());
        // check that all wall IDs are <= 0xFF
        let wallsok = tiles.iter().all(|w| *w <= 0xFF);
        assert!(wallsok);
        // check that all thing IDs are <= 0x1FF
        let thingsok = things.iter().all(|t| *t <= 0x1FF);
        assert!(thingsok);

        Self {
            name,
            width,
            height,
            tiles,
            things,
        }
    }

    #[inline]
    pub fn tile(&self, x: i32, y: i32) -> u16 {
        self.safe_item_from_array(x, y, &self.tiles)
    }

    #[inline]
    pub fn thing(&self, x: i32, y: i32) -> u16 {
        self.safe_item_from_array(x, y, &self.things)
    }

    fn safe_item_from_array(&self, x: i32, y: i32, vect: &Vec<u16>) -> u16 {
        let w = self.width as i32;
        let h = self.height as i32;
        if x >= 0 && y >= 0 && x < w && y < h {
            let idx = (y * w + x) as usize;
            vect[idx]
        } else {
            0
        }
    }
}
