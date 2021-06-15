mod enemy_group;
mod font;
mod game;
mod music;
mod ppu;
mod rom;
mod spawn_table;

pub use crate::enemy_group::*;
pub use crate::font::*;
pub use crate::game::*;
pub use crate::music::*;
pub use crate::ppu::*;
pub use crate::rom::*;
pub use crate::spawn_table::*;

pub const OBJECT_NAME: [&str; 0x29] = [
    "",
    "レウス",
    "テュラ",
    "エイク",
    "ソレル",
    "ディダ",
    "ペンド",
    "リアード",
    "バタフ",
    "スラント",
    "カルゴ",
    "アトリス",
    "メルス",
    "プリング",
    "ヤール",
    "ビーグ",
    "メーバ",
    "ルイド",
    "ジェラ",
    "ルダン",
    "リューク",
    "ビータ",
    "テミス",
    "パトラ",
    "ドラク",
    "プリズン",
    "カディス",
    "ステリア",
    "リーデ",
    "グハ",
    "ジェリコ",
    "ラザロ",
    "",
    "ソープラー",
    "スターブレインのコア",
    "スターブレインの砲台",
    "パワーアップアイテム",
    "空中物の大爆発",
    "地上物の大爆発",
    "地上物の小爆発",
    "空中物の小爆発",
];
