//! Maintains the game status - points, ammo, health, various toggles etc.
//! Also knows how to paint the status bar.

use crate::*;

pub struct GameStatus(Vec<i32>);

impl GameStatus {
    pub fn new(episode: i32) -> Self {
        let mut status = Self(vec![0; VEC_LENGTH]);
        status.0[EPISODE] = episode;
        status.0[LIVES] = 3;
        status.0[HEALTH] = 100;
        status.0[AMMO] = 8;
        status.0[FLAGS] = 1; // selected weapon = pistol
        status
    }

    pub fn set_floor(&mut self, floor: i32, kills: i32, secrets: i32, treasures: i32) {
        self.0[FLAGS] &= FLAGS_KEPT_BETWEEN_FLOORS;
        self.0[FLOOR] = floor;
        self.0[CNT_KILLS] = 0;
        self.0[CNT_SECRETS] = 0;
        self.0[CNT_TREASURES] = 0;
        self.0[TOTAL_KILLS] = kills;
        self.0[TOTAL_SECRETS] = secrets;
        self.0[TOTAL_TREASURES] = treasures;
    }

    #[inline]
    pub fn increment_kills(&mut self) {
        self.0[CNT_KILLS] += 1;
    }

    #[inline]
    pub fn increment_secrets(&mut self) {
        self.0[CNT_SECRETS] += 1;
    }

    #[inline]
    pub fn increment_treasures(&mut self) {
        self.0[CNT_TREASURES] += 1;
    }

    #[inline]
    pub fn increment_score(&mut self, delta_score: i32) {
        self.0[SCORE] += delta_score;
    }

    #[inline]
    pub fn try_select_weapon(&mut self, weapon: i32) -> bool {
        assert!(weapon >= 0 && weapon <= 3);
        if weapon == 2 && (self.0[FLAGS] & FLG_HAS_MACHINE_GUN) == 0 {
            false
        } else if weapon == 3 && (self.0[FLAGS] & FLG_HAS_CHAIN_GUN) == 0 {
            false
        } else {
            self.0[FLAGS] = (self.0[FLAGS] & !SEL_WEAPON_MASK) | weapon;
            true
        }
    }

    #[inline]
    pub fn selected_weapon(&self) -> i32 {
        self.0[FLAGS] & SEL_WEAPON_MASK
    }

    // TODO - one method to handle pick-ups:
    // => returns bool: true if item can be picked, false otherwise (e.g. health item when health is full)
    //  - health, ammo, treasure, 1up etc
    //  - ammo
    #[inline]
    pub fn try_pick_up_ammo(&mut self, cnt_ammo: i32) -> bool {
        if self.0[AMMO] < 99 {
            self.0[AMMO] = Ord::min(99, self.0[AMMO] + cnt_ammo);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn try_consume_ammo(&mut self) -> bool {
        if self.0[AMMO] > 0 {
            self.0[AMMO] -= 1;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        self.0[HEALTH] <= 0
    }

    #[inline]
    pub fn try_decrement_lives(&mut self) -> bool {
        if self.0[LIVES] > 0 {
            self.0[LIVES] -= 1;
            self.0[HEALTH] = 100;
            true
        } else {
            false
        }
    }

    pub fn paint_status_bar(&self, scrbuf: &mut ScreenBuffer, assets: &GameAssets) {
        const LIGHT_BG: u8 = 125;
        const DARK_BG: u8 = 127;

        // paint background + check if enabled
        let y = scrbuf.view_height();
        let scrh = scrbuf.scr_height();
        let barh = scrh - y;
        if barh <= 0 {
            return;
        }
        let w = scrbuf.scr_width();
        let th = 1 + (scrh / 250); // thickness of light bars
        scrbuf.fill_rect(0, y, w, th, LIGHT_BG);
        scrbuf.fill_rect(0, y + th, w, barh - th, DARK_BG);

        // display main info
        // TODO just a hack for now => improve it !!!
        let str = format!(
            "Ep.{} floor {}    Score: {}",
            self.0[EPISODE] + 1,
            self.0[FLOOR] + 1,
            self.0[SCORE]
        );
        assets.font2.draw_text(6, y + 6, &str, 15, scrbuf);

        let str = format!(
            "Health: {}   Ammo: {}   Lives: {}",
            self.0[HEALTH], self.0[AMMO], self.0[LIVES]
        );
        assets.font2.draw_text(6, y + 26, &str, 14, scrbuf);

        let str = format!(
            "SelWpn: {}     MchGun-{}  ChnGun-{}  BlueKey-{}  GoldKey-{}",
            self.selected_weapon(),
            _yesno(self.0[FLAGS], FLG_HAS_MACHINE_GUN),
            _yesno(self.0[FLAGS], FLG_HAS_CHAIN_GUN),
            _yesno(self.0[FLAGS], FLG_HAS_BLUE_KEY),
            _yesno(self.0[FLAGS], FLG_HAS_GOLD_KEY),
        );
        assets.font1.draw_text(6, y + 46, &str, 11, scrbuf);

        let str = format!(
            "K: {}/{}   S: {}/{}   T: {}/{}",
            self.0[CNT_KILLS],
            self.0[TOTAL_KILLS],
            self.0[CNT_SECRETS],
            self.0[TOTAL_SECRETS],
            self.0[CNT_TREASURES],
            self.0[TOTAL_TREASURES]
        );
        assets.font1.draw_text(6, y + 62, &str, 24, scrbuf);

        // TODO temporary
        _temp_slideshow(assets, scrbuf, y, w);
    }
}

//-------------------------------
//  Internal stuff

const FLAGS: usize = 0;
const EPISODE: usize = 1;
const FLOOR: usize = 2;
const SCORE: usize = 3;
const LIVES: usize = 4;
const HEALTH: usize = 5;
const AMMO: usize = 6;
const CNT_KILLS: usize = 7;
const CNT_SECRETS: usize = 8;
const CNT_TREASURES: usize = 9;
const TOTAL_KILLS: usize = 10;
const TOTAL_SECRETS: usize = 11;
const TOTAL_TREASURES: usize = 12;
const VEC_LENGTH: usize = 13;

const SEL_WEAPON_MASK: i32 = 0x07;
const FLG_HAS_MACHINE_GUN: i32 = 1 << 3;
const FLG_HAS_CHAIN_GUN: i32 = 1 << 4;
const FLG_HAS_BLUE_KEY: i32 = 1 << 5;
const FLG_HAS_GOLD_KEY: i32 = 1 << 6;
const FLAGS_KEPT_BETWEEN_FLOORS: i32 = SEL_WEAPON_MASK | FLG_HAS_MACHINE_GUN | FLG_HAS_CHAIN_GUN;

//-------------------

static mut TMP_TIMER: f64 = 0.0;
static mut TMP_INDEX: usize = 0;

fn _yesno(x: i32, flag: i32) -> &'static str {
    if x & flag != 0 {
        "YES"
    } else {
        "no"
    }
}

pub fn _temp_advance_fwd() {
    _temp_timer_update(0.5);
}

pub fn _temp_advance_back() {
    _temp_timer_update(499.5);
}

fn _temp_timer_update(elapsed: f64) {
    unsafe {
        let new_time = TMP_TIMER + elapsed * 2.0;
        let i = new_time.floor() as usize;
        TMP_TIMER = new_time - (i as f64);
        TMP_INDEX = (TMP_INDEX + i) % 1000;
    }
}

// TODO temporary paint graphics
fn _temp_slideshow(assets: &GameAssets, scrbuf: &mut ScreenBuffer, y: i32, w: i32) {
    // fake update - dependent on FPS, but it's ok for now
    //_temp_timer_update(1.0 / 320.0);

    let tidx;
    unsafe {
        tidx = TMP_INDEX;
    }
    _temp_paint_pic(w - 80, y + 6, tidx, &assets.walls, "WALL", assets, scrbuf);
    _temp_paint_pic(w - 170, y + 6, tidx, &assets.sprites, "SPRT", assets, scrbuf);

    // paint pics
    let piclen = PicDict::pic_count();
    let picidx = tidx % PicDict::pic_count();
    let picenum = PicType::from_repr(picidx).unwrap();
    let pic = assets.pics.pic_by_index(picidx);
    let (sw, _) = pic.size();
    //let width = Ord::min(sw as i32, 128);
    let width = sw as i32;
    scrbuf.draw_scaled_pic(w - 320, 20, width, pic);
    let str = format!("{picenum} {picidx}/{piclen}");
    assets.font1.draw_text(w - 320, 6, &str, 14, scrbuf);
}

fn _temp_paint_pic(
    x: i32,
    y: i32,
    tidx: usize,
    gfx: &[GfxData],
    msg: &str,
    assets: &GameAssets,
    scrbuf: &mut ScreenBuffer,
) {
    let w = 64;
    let len = gfx.len();
    let sprtidx = tidx % len;
    let sprite = &gfx[sprtidx];
    let (sw, _) = sprite.size();
    let width = Ord::min(sw as i32, w);
    scrbuf.draw_scaled_pic(x, y + 14, width, sprite);
    let str = format!("{msg} {sprtidx}/{len}");
    assets.font1.draw_text(x, y, &str, 14, scrbuf);
}
