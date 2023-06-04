//! Various notifications + sound effects.

use std::collections::VecDeque;

use crate::{Collectible, GameAssets, ScreenBuffer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Notification {
    LockNeedsGoldKey,
    LockNeedsSilverKey,
    LockNoKeyAvailable,
    FoundSecret,
    GotAllKils,
    GotAllSecrets,
    GotAllTreasures,
    // TODO add notifications for ALL possible sounds (+ temporary messages)
}

pub struct Notifier(VecDeque<TimedMessage>);

impl Notifier {
    pub fn new() -> Self {
        Self(VecDeque::with_capacity(MAX_MESSAGES))
    }

    pub fn notify(&mut self, note: Notification) {
        // TODO sounds !!
        let msg = from_notification(note).to_string();
        self.add_message(msg);
    }

    pub fn notify_collectible(&mut self, coll: Collectible) {
        // TODO sounds !!
        let msg = from_collectible(coll).to_string();
        self.add_message(msg);
    }

    pub fn update_time(&mut self, elapsed: f64) {
        self.0.iter_mut().for_each(|notif| {
            notif.timeout -= elapsed;
        });
        // remove old, finished notifications
        while let Some(notif) = self.0.back() {
            if notif.timeout <= 0.0 {
                self.0.pop_back();
            } else {
                break;
            }
        }
    }

    pub fn paint(&self, scrbuf: &mut ScreenBuffer, assets: &GameAssets) {
        let mut y = 5;
        for tm in self.0.iter() {
            assets.font1.draw_text(6, y + 1, &tm.msg, 0, scrbuf);
            assets.font1.draw_text(5, y, &tm.msg, 15, scrbuf);
            y += 15;
        }
    }

    //------------------

    fn add_message(&mut self, msg: String) {
        if msg.is_empty() {
            return;
        }
        // remove old notifications
        while self.0.len() >= MAX_MESSAGES {
            self.0.pop_back();
        }
        // add new notification
        let tm = TimedMessage {
            msg,
            timeout: MSG_TIMEOUT,
        };
        self.0.push_front(tm);
    }
}

//---------------------------
//  Internal stuff

const MSG_TIMEOUT: f64 = 5.0;
const MAX_MESSAGES: usize = 5;

#[derive(Clone)]
struct TimedMessage {
    msg: String,
    timeout: f64,
}

fn from_notification(note: Notification) -> &'static str {
    match note {
        Notification::LockNeedsGoldKey => "Locked - you need a GOLD key",
        Notification::LockNeedsSilverKey => "Locked - you need a SILVER key",
        Notification::LockNoKeyAvailable => "Locked, and there is no key for it :(",
        Notification::FoundSecret => "You have found a secret :)",
        Notification::GotAllKils => "You have killed everybody",
        Notification::GotAllSecrets => "You have found all the secrets",
        Notification::GotAllTreasures => "You have found all the treasures",
    }
}

fn from_collectible(coll: Collectible) -> &'static str {
    match coll {
        Collectible::DogFood => "You ate some dog food :(",
        Collectible::GoodFood => "You ate a tasty meal",
        Collectible::FirstAid => "You found a first-aid kit",
        Collectible::Gibs1 | Collectible::Gibs2 => "YUCK - you ate some gibs :(((",
        Collectible::AmmoClipSmall | Collectible::AmmoClipNormal => "You found some ammo",
        Collectible::MachineGun => "You found a machine gun",
        Collectible::ChainGun => "YEAH - you found the Gatling Gun :D",
        Collectible::GoldKey => "You found a GOLD key",
        Collectible::SilverKey => "You found a SILVER key",
        Collectible::TreasureCross
        | Collectible::TreasureCup
        | Collectible::TreasureChest
        | Collectible::TreasureCrown => "You found some treasure",
        Collectible::TreasureOneUp => "You found a Megasphere :]",
        Collectible::AmmoBox => "You found a box of ammo",
        Collectible::SpearOfDestiny => "You got the SPEAR OF DESTINY",
        _ => "",
    }
}
