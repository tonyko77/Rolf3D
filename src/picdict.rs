//! Picture dictionary - for accessing the PIC-s.
//! Since their indices are game-dependent, this module knows where each pic is located for each game type.

use crate::GfxData;

//#[derive(Clone, Copy, PartialEq, Eq)]
#[derive(Clone, Copy, PartialEq, Eq)] //, strum_macros::Display, strum_macros::FromRepr)]
pub enum PicType {
    BackDropScreen,
    //---
    OptionTitle,
    Cursor1,
    Cursor2,
    SelectionOff,
    SelectionOn,
    SoundFxTitle,
    DigiSoundTitle,
    MusicTitle,
    MoveSelBack,
    DifficultyBaby,
    DifficultyEasy,
    DifficultyMedium,
    DifficultyHard,
    LoadSaveFloppy,
    DiskLoading1,
    DiskLoading2,
    ControlTitle,
    CustomizeTitle,
    LoadGameTitle,
    SaveGameTitle,
    //---
    Episode1,
    Episode2,
    Episode3,
    Episode4,
    Episode5,
    Episode6,
    //---
    FloorEndGuy,
    FloorEndColon,
    FloorEndNum0,
    FloorEndNum1,
    FloorEndNum2,
    FloorEndNum3,
    FloorEndNum4,
    FloorEndNum5,
    FloorEndNum6,
    FloorEndNum7,
    FloorEndNum8,
    FloorEndNum9,
    FloorEndPercent,
    FloorEndLetterA,
    FloorEndLetterB,
    FloorEndLetterC,
    FloorEndLetterD,
    FloorEndLetterE,
    FloorEndLetterF,
    FloorEndLetterG,
    FloorEndLetterH,
    FloorEndLetterI,
    FloorEndLetterJ,
    FloorEndLetterK,
    FloorEndLetterL,
    FloorEndLetterM,
    FloorEndLetterN,
    FloorEndLetterO,
    FloorEndLetterP,
    FloorEndLetterQ,
    FloorEndLetterR,
    FloorEndLetterS,
    FloorEndLetterT,
    FloorEndLetterU,
    FloorEndLetterV,
    FloorEndLetterW,
    FloorEndLetterX,
    FloorEndLetterY,
    FloorEndLetterZ,
    FloorEndExclamation,
    FloorEndApostrophe,
    FloorEndGuy2,
    FloorEndWinnerBJ,
    //-------
    StatusBar,
    TitleScreen,
    Pg13Pic,
    CreditsScreen,
    HighScoresTitle,
    //-------
    StatusKnife,
    StatusPistol,
    StatusMachineGun,
    StatusChainGun,
    StatusNoKey,
    StatusGoldKey,
    StatusSilverKey,
    StatusNumBlank,
    StatusNum0,
    StatusNum1,
    StatusNum2,
    StatusNum3,
    StatusNum4,
    StatusNum5,
    StatusNum6,
    StatusNum7,
    StatusNum8,
    StatusNum9,
    StatusFace1A,
    StatusFace1B,
    StatusFace1C,
    StatusFace2A,
    StatusFace2B,
    StatusFace2C,
    StatusFace3A,
    StatusFace3B,
    StatusFace3C,
    StatusFace4A,
    StatusFace4B,
    StatusFace4C,
    StatusFace5A,
    StatusFace5B,
    StatusFace5C,
    StatusFace6A,
    StatusFace6B,
    StatusFace6C,
    StatusFace7A,
    StatusFace7B,
    StatusFace7C,
    StatusFaceDead,
    StatusFaceGatling,
    StatusFaceGod1,
    StatusFaceGod2,
    StatusFaceGod3,
    //-------
    Paused,
    GetPsyched,
}

pub struct PicDict(Vec<GfxData>);

impl PicDict {
    #[inline]
    pub fn new(game_ext: &str, input: Vec<GfxData>) -> Self {
        let mapped_vec = into_ordered_pics_vec(game_ext, input);
        Self(mapped_vec)
    }

    #[inline]
    pub fn pic(&self, typ: PicType, delta_idx: usize) -> &GfxData {
        let idx = (typ as usize) + delta_idx;
        assert!(idx < self.0.len());
        &self.0[idx]
    }

    #[inline]
    pub fn pic_by_index(&self, idx: usize) -> &GfxData {
        assert!(idx < self.0.len());
        &self.0[idx]
    }

    #[inline]
    pub fn pic_count() -> usize {
        TOTAL_PICS
    }
}

//--------------------------------
//  Internal stuff

const BACK_DROP: usize = PicType::BackDropScreen as usize;
//-------
const OPTION_TITLE: usize = PicType::OptionTitle as usize;
const SAVE_TITLE: usize = PicType::SaveGameTitle as usize;
const EPISODE_6: usize = PicType::Episode6 as usize;
//-------
const FLOOR_END_GUY: usize = PicType::FloorEndGuy as usize;
const FLOOR_END_WINNER: usize = PicType::FloorEndWinnerBJ as usize;
//-------
const STATUS_BAR: usize = PicType::StatusBar as usize;
const TITLE_SCREEN: usize = PicType::TitleScreen as usize;
const PG13_PIC: usize = PicType::Pg13Pic as usize;
const CREDITS_SCREEN: usize = PicType::CreditsScreen as usize;
const HIGH_SCORES: usize = PicType::HighScoresTitle as usize;
//-------
const STATUS_KNIFE: usize = PicType::StatusKnife as usize;
const STATUS_GOD1: usize = PicType::StatusFaceGod1 as usize;
const STATUS_GOD2: usize = PicType::StatusFaceGod2 as usize;
const STATUS_GOD3: usize = PicType::StatusFaceGod3 as usize;
//-------
const PAUSED: usize = PicType::Paused as usize;
const GET_PSYCHED: usize = PicType::GetPsyched as usize;
const TOTAL_PICS: usize = GET_PSYCHED + 1;
const BAD_IDX: usize = 0xFFFF;

/// Move the pics from the input vector into a new vector,
/// so that they are orderec exactly like in the `PicType` enum.
fn into_ordered_pics_vec(game_ext: &str, input: Vec<GfxData>) -> Vec<GfxData> {
    let mut mapped_vec = vec![GfxData::new_empty(); TOTAL_PICS];
    // get translation func
    let transl_func = match game_ext {
        "WL1" => translate_wolf1,
        "WL3" => translate_wolf3or6,
        "WL6" => translate_wolf3or6,
        "SDM" => translate_sdm,
        _ => translate_sod,
    };
    // map inputs to outputs
    let mut in_to_out = vec![BAD_IDX; input.len()];
    for out_idx in 0..TOTAL_PICS {
        let in_idx = transl_func(out_idx);
        if in_idx < BAD_IDX {
            in_to_out[in_idx] = out_idx;
        }
    }
    // convert input vector
    let mut idx = 0;
    for pic in input.into_iter() {
        let out_idx = in_to_out[idx];
        idx += 1;
        if out_idx < BAD_IDX {
            mapped_vec[out_idx] = pic;
        }
    }
    // take care of gaps, in case of WLx
    let ch = game_ext.bytes().next().unwrap_or(0);
    if ch == ('W' as u8) {
        mapped_vec[STATUS_GOD2] = mapped_vec[STATUS_GOD1].clone();
        mapped_vec[STATUS_GOD3] = mapped_vec[STATUS_GOD1].clone();
        mapped_vec[BACK_DROP] = convert_w3d_title_screen_to_back_drop(&mapped_vec[TITLE_SCREEN]);
    }
    mapped_vec
}

// see GFXV_WL1.H
fn translate_wolf1(enum_idx: usize) -> usize {
    match enum_idx {
        OPTION_TITLE..=EPISODE_6 => enum_idx + 18,
        FLOOR_END_GUY..=STATUS_GOD1 => enum_idx + 52 - FLOOR_END_GUY,
        PAUSED => 142,
        GET_PSYCHED => 143,
        _ => BAD_IDX,
    }
}

// see GFXV_WL6.H
fn translate_wolf3or6(enum_idx: usize) -> usize {
    match enum_idx {
        OPTION_TITLE..=EPISODE_6 => enum_idx + 6,
        FLOOR_END_GUY..=STATUS_GOD1 => enum_idx + 40 - FLOOR_END_GUY,
        PAUSED => 130,
        GET_PSYCHED => 131,
        _ => BAD_IDX,
    }
}

fn translate_sdm(enum_idx: usize) -> usize {
    match enum_idx {
        FLOOR_END_GUY..=FLOOR_END_WINNER => enum_idx + 28 - FLOOR_END_GUY,
        STATUS_BAR => 73,
        TITLE_SCREEN => 71,
        PG13_PIC => 74,
        CREDITS_SCREEN => 75,
        STATUS_KNIFE..=STATUS_GOD3 => enum_idx + 76 - STATUS_KNIFE,
        PAUSED => 123,
        GET_PSYCHED => 124,
        _ => translate_spear_common(enum_idx),
    }
}

fn translate_sod(enum_idx: usize) -> usize {
    match enum_idx {
        FLOOR_END_GUY..=FLOOR_END_WINNER => enum_idx + 33 - FLOOR_END_GUY,
        STATUS_BAR => 87,
        TITLE_SCREEN => 79,
        PG13_PIC => 88,
        CREDITS_SCREEN => 89,
        STATUS_KNIFE..=STATUS_GOD3 => enum_idx + 98 - STATUS_KNIFE,
        PAUSED => 145,
        GET_PSYCHED => 146,
        _ => translate_spear_common(enum_idx),
    }
}

// TODO also fix title screen for SOD !!!
fn translate_spear_common(enum_idx: usize) -> usize {
    match enum_idx {
        BACK_DROP => 0,
        HIGH_SCORES => 26,
        // TODO FIX THESE !!!
        // (the option pics are BAD, but I'm too tired to fix them :/ )
        OPTION_TITLE..=SAVE_TITLE => enum_idx + 1,
        _ => BAD_IDX,
    }
}

fn convert_w3d_title_screen_to_back_drop(input: &GfxData) -> GfxData {
    // TODO red scale - maybe it looks better ?!
    // let mut reds = vec![];
    // for i in 0..16 {
    //     reds.push(47 - i);
    // }
    // for i in 0..8 {
    //     reds.push(55 - i);
    // }
    let mut grays = vec![0];
    for i in 0..17 {
        grays.push(31 - i);
    }

    let mut pic = input.clone();
    pic.grayscale(&grays);
    pic
}
