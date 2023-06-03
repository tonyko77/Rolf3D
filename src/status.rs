//! Maintains the game status - points, ammo, health, various toggles etc.
//! Also knows how to paint the status bar.

use crate::*;

const MAX_HEALTH: i32 = 100;
const MAX_AMMO: i32 = 99;
pub struct GameStatus(Vec<i32>);

// TODO (later) check which of these methods are actually needed for gameplay
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

    pub fn set_floor(&mut self, floor: i32, cnt_enemies: i32) {
        self.0[FLAGS] &= FLAGS_KEPT_BETWEEN_FLOORS;
        self.0[FLOOR] = floor;
        self.0[CNT_KILLS] = 0;
        self.0[CNT_SECRETS] = 0;
        self.0[CNT_TREASURES] = 0;
        self.0[TOTAL_KILLS] = cnt_enemies;
        self.0[TOTAL_SECRETS] = 0;
        self.0[TOTAL_TREASURES] = 0;
    }

    pub fn read_floor_cell(&mut self, cell: &MapCell) {
        if cell.is_push_wall() {
            self.0[TOTAL_SECRETS] += 1;
        }
        match cell.collectible() {
            Collectible::TreasureCross
            | Collectible::TreasureCup
            | Collectible::TreasureChest
            | Collectible::TreasureCrown
            | Collectible::TreasureOneUp => {
                self.0[TOTAL_TREASURES] += 1;
            }
            _ => {}
        }
    }

    // TODO temporary !!
    pub fn _tmp_give_stuff(&mut self) {
        self.0[FLAGS] |= FLG_HAS_GOLD_KEY | FLG_HAS_SILVER_KEY | FLG_HAS_MACHINE_GUN | FLG_HAS_CHAIN_GUN;
    }

    #[inline]
    pub fn increment_kills(&mut self, kill_score: i32) {
        self.0[CNT_KILLS] += 1;
        self.0[SCORE] += kill_score;
    }

    #[inline]
    pub fn found_secret(&mut self) {
        self.0[CNT_SECRETS] += 1;
    }

    #[inline]
    pub fn try_select_weapon(&mut self, weapon: i32) {
        assert!(weapon >= 0 && weapon <= 3);
        let has_ammo = (weapon == 0) || self.0[AMMO] > 0;
        if has_ammo && self.has_weapon(weapon) {
            self.0[FLAGS] = (self.0[FLAGS] & !SEL_WEAPON_MASK) | weapon;
        }
    }

    // 0 = no key, 1 = gold, 2 = silver
    #[inline]
    pub fn has_key(&self, key: u8) -> bool {
        match key {
            0 => true,
            1 => self.has_flag(FLG_HAS_GOLD_KEY),
            2 => self.has_flag(FLG_HAS_SILVER_KEY),
            _ => false,
        }
    }

    // TODO is this needed?
    // #[inline]
    // pub fn has_ammo(&self) -> bool {
    //     self.0[AMMO] > 0
    // }

    #[inline]
    pub fn consume_ammo(&mut self) {
        self.update_ammo(-1);
        if self.0[AMMO] <= 0 {
            // no more ammo => switch to knife
            self.try_select_weapon(0);
        }
    }

    #[inline]
    pub fn consume_health(&mut self, damage: i32) {
        self.update_health(-damage);
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

    pub fn can_consume(&self, coll: Collectible) -> bool {
        match coll {
            Collectible::None => false,
            Collectible::DogFood | Collectible::GoodFood | Collectible::FirstAid => self.0[HEALTH] < MAX_HEALTH,
            Collectible::AmmoClipSmall | Collectible::AmmoClipNormal | Collectible::AmmoBox => self.0[AMMO] < MAX_AMMO,
            Collectible::MachineGun => !self.has_flag(FLG_HAS_MACHINE_GUN),
            Collectible::ChainGun => !self.has_flag(FLG_HAS_CHAIN_GUN),
            Collectible::GoldKey => !self.has_flag(FLG_HAS_GOLD_KEY),
            Collectible::SilverKey => !self.has_flag(FLG_HAS_SILVER_KEY),
            _ => true,
        }
    }

    pub fn consume(&mut self, coll: Collectible) {
        match coll {
            Collectible::DogFood => {
                self.update_health(4);
            }
            Collectible::GoodFood => {
                self.update_health(10);
            }
            Collectible::FirstAid => {
                self.update_health(25);
            }
            Collectible::AmmoClipSmall => {
                self.update_ammo(4);
            }
            Collectible::AmmoClipNormal => {
                self.update_ammo(8);
            }
            Collectible::AmmoBox => {
                self.update_ammo(25);
            }
            Collectible::MachineGun => {
                self.0[FLAGS] |= FLG_HAS_MACHINE_GUN;
                self.update_ammo(6);
                if self.0[FLAGS] & (FLG_HAS_MACHINE_GUN | FLG_HAS_CHAIN_GUN) == 0 {
                    // new best weapon
                    self.try_select_weapon(2);
                }
            }
            Collectible::ChainGun => {
                self.0[FLAGS] |= FLG_HAS_CHAIN_GUN;
                self.update_ammo(6);
                if self.0[FLAGS] & FLG_HAS_CHAIN_GUN == 0 {
                    // new best weapon
                    self.try_select_weapon(3);
                }
            }
            Collectible::GoldKey => {
                self.0[FLAGS] |= FLG_HAS_GOLD_KEY;
            }
            Collectible::SilverKey => {
                self.0[FLAGS] |= FLG_HAS_SILVER_KEY;
            }
            Collectible::TreasureCross => {
                self.0[SCORE] += 100;
                self.0[CNT_TREASURES] += 1;
            }
            Collectible::TreasureCup => {
                self.0[SCORE] += 500;
                self.0[CNT_TREASURES] += 1;
            }
            Collectible::TreasureChest => {
                self.0[SCORE] += 1000;
                self.0[CNT_TREASURES] += 1;
            }
            Collectible::TreasureCrown => {
                self.0[SCORE] += 5000;
                self.0[CNT_TREASURES] += 1;
            }
            Collectible::TreasureOneUp => {
                self.update_health(100);
                self.update_ammo(25);
                self.0[LIVES] += 1;
                self.0[CNT_TREASURES] += 1;
            }
            Collectible::SpearOfDestiny => {
                // SOD only
                todo!("What should I do with the Spear of Destiny ?");
            }
            _ => {}
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
            "Wpn:{}   MchG:{}  ChnG:{}  SilvK:{}  GoldK:{}",
            self.selected_weapon(),
            _yesno(self.0[FLAGS], FLG_HAS_MACHINE_GUN),
            _yesno(self.0[FLAGS], FLG_HAS_CHAIN_GUN),
            _yesno(self.0[FLAGS], FLG_HAS_SILVER_KEY),
            _yesno(self.0[FLAGS], FLG_HAS_GOLD_KEY),
        );
        assets.font1.draw_text(6, y + 46, &str, 11, scrbuf);

        let str = self.get_secrets_msg();
        assets.font1.draw_text(6, y + 62, &str, 24, scrbuf);

        // TODO temporary
        _temp_slideshow(assets, scrbuf, y, w);
    }

    #[inline]
    pub fn get_secrets_msg(&self) -> String {
        format!(
            "K: {}/{}   S: {}/{}   T: {}/{}",
            self.0[CNT_KILLS],
            self.0[TOTAL_KILLS],
            self.0[CNT_SECRETS],
            self.0[TOTAL_SECRETS],
            self.0[CNT_TREASURES],
            self.0[TOTAL_TREASURES]
        )
    }

    #[inline]
    fn has_weapon(&self, weapon: i32) -> bool {
        match weapon {
            0 | 1 => true,
            2 => self.has_flag(FLG_HAS_MACHINE_GUN),
            3 => self.has_flag(FLG_HAS_CHAIN_GUN),
            _ => false,
        }
    }

    #[inline]
    fn selected_weapon(&self) -> i32 {
        self.0[FLAGS] & SEL_WEAPON_MASK
    }

    #[inline]
    fn update_health(&mut self, health_update: i32) {
        self.0[HEALTH] = (self.0[HEALTH] + health_update).clamp(0, MAX_HEALTH);
    }

    #[inline]
    fn update_ammo(&mut self, ammo_update: i32) {
        let was_empty = self.0[AMMO] == 0;
        self.0[AMMO] = (self.0[AMMO] + ammo_update).clamp(0, MAX_AMMO);
        if was_empty && self.0[AMMO] > 0 {
            // got ammo => switch to best weapon
            let best_weapon = if self.has_flag(FLG_HAS_CHAIN_GUN) {
                3
            } else if self.has_flag(FLG_HAS_MACHINE_GUN) {
                2
            } else {
                1
            };
            self.try_select_weapon(best_weapon);
        }
    }

    #[inline]
    fn has_flag(&self, flag: i32) -> bool {
        self.0[FLAGS] & flag != 0
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
const FLG_HAS_SILVER_KEY: i32 = 1 << 5;
const FLG_HAS_GOLD_KEY: i32 = 1 << 6;
const FLAGS_KEPT_BETWEEN_FLOORS: i32 = SEL_WEAPON_MASK | FLG_HAS_MACHINE_GUN | FLG_HAS_CHAIN_GUN;

//-------------------

static mut TMP_TIMER: f64 = 0.0;
static mut TMP_INDEX: usize = 0;

fn _yesno(x: i32, flag: i32) -> &'static str {
    if x & flag != 0 {
        "Y"
    } else {
        "N"
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
    let width = Ord::min(sw as i32, 128);
    scrbuf.draw_scaled_pic(w - 320, y + 20, width, pic);
    let str = format!("{picenum} {picidx}/{piclen}");
    assets.font1.draw_text(w - 320, y + 6, &str, 14, scrbuf);
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
