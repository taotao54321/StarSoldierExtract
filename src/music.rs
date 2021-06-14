use std::convert::TryFrom;

use byteorder::{ByteOrder, LE};

use crate::rom::*;

// BGM ID 10 はただの無音なので無視する。
const MUSIC_COUNT: usize = 9;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SquareDuty {
    Eighth,
    Quarter,
    Half,
    QuarterNeg,
}

impl SquareDuty {
    pub fn new(value: u8) -> Self {
        match value {
            0 => Self::Eighth,
            1 => Self::Quarter,
            2 => Self::Half,
            3 => Self::QuarterNeg,
            _ => panic!("invalid square duty value: {}", value),
        }
    }

    pub fn value(self) -> u8 {
        match self {
            Self::Eighth => 0,
            Self::Quarter => 1,
            Self::Half => 2,
            Self::QuarterNeg => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MusicCommand {
    Tone { octave: u8, note: u8 },
    Rest,
    SetLength { length: u8 },
    LoopBegin { count: u8 },
    LoopEnd,
    Restart,
    End,
}

impl MusicCommand {
    pub fn new_tone(octave: u8, note: u8) -> Self {
        assert!((0..=11).contains(&note));
        Self::Tone { octave, note }
    }

    pub fn new_rest() -> Self {
        Self::Rest
    }

    pub fn new_set_length(length: u8) -> Self {
        assert!(length > 0);
        Self::SetLength { length }
    }

    pub fn new_loop_begin(count: u8) -> Self {
        assert!(count > 0);
        Self::LoopBegin { count }
    }

    pub fn new_loop_end() -> Self {
        Self::LoopEnd
    }

    pub fn new_restart() -> Self {
        Self::Restart
    }

    pub fn new_end() -> Self {
        Self::End
    }
}

#[derive(Debug)]
pub struct Music {
    pub id: u8,
    pub sq_envelope: u8,
    pub sq_duty: SquareDuty,
    pub track_sq1: Vec<MusicCommand>,
    pub track_sq2: Vec<MusicCommand>,
    pub track_tri: Vec<MusicCommand>,
}

impl Music {
    pub fn write_mml<W: std::io::Write>(&self, mut wtr: W) -> eyre::Result<()> {
        // FlMML reference: https://gist.github.com/anonymous/975e4cf634c2b156621e662b5fd12e4a

        // ゲーム内では音長をフレーム単位で扱っているが、MMLでは384分音符が1単位となる。
        // そこで、192分音符=1F とすると、4分音符=48F より、BPMは 3600/48=75 となる。

        writeln!(wtr, "T75")?;

        Self::write_mml_sq_track(&mut wtr, self.sq_envelope, self.sq_duty, &self.track_sq1)?;
        Self::write_mml_sq_track(&mut wtr, self.sq_envelope, self.sq_duty, &self.track_sq2)?;
        Self::write_mml_tri_track(&mut wtr, &self.track_tri)?;

        Ok(())
    }

    fn write_mml_sq_track<W: std::io::Write>(
        wtr: &mut W,
        envelope: u8,
        duty: SquareDuty,
        track: &[MusicCommand],
    ) -> eyre::Result<()> {
        // APU のエンベロープの周期は t=(envelope+1)/240 秒。
        // エンベロープありの場合、初期音量は 15 なので、無音になるまで 15*t 秒かかる。
        // これを x/127 秒に補正する。
        let decay = {
            let t = f64::from(envelope + 1) / 240.0;
            (127.0 * 15.0 * t).round() as u32
        };

        writeln!(
            wtr,
            "@5@W{} V15 @E1,0,{},0,0 ",
            match duty {
                SquareDuty::Eighth => 1,
                SquareDuty::Quarter => 2,
                SquareDuty::Half => 4,
                SquareDuty::QuarterNeg => 6,
            },
            decay
        )?;

        Self::write_mml_track(wtr, track)?;

        writeln!(wtr, ";")?;

        Ok(())
    }

    fn write_mml_tri_track<W: std::io::Write>(
        wtr: &mut W,
        track: &[MusicCommand],
    ) -> eyre::Result<()> {
        writeln!(wtr, "V1 @6")?;

        Self::write_mml_track(wtr, track)?;

        writeln!(wtr, ";")?;

        Ok(())
    }

    fn write_mml_track<W: std::io::Write>(wtr: &mut W, track: &[MusicCommand]) -> eyre::Result<()> {
        let mut length_cur = None;

        for &cmd in track {
            match cmd {
                MusicCommand::Tone { octave, note } => {
                    write!(
                        wtr,
                        "O{}{}%{} ",
                        octave,
                        Self::note_to_str(note),
                        Self::length_to_tick(length_cur.expect("length_cur not set"))
                    )?;
                }
                MusicCommand::Rest => {
                    write!(
                        wtr,
                        "R%{} ",
                        Self::length_to_tick(length_cur.expect("length_cur not set"))
                    )?;
                }
                MusicCommand::SetLength { length } => length_cur = Some(u32::from(length)),
                MusicCommand::LoopBegin { count } => write!(wtr, "/:{} ", count)?,
                MusicCommand::LoopEnd => write!(wtr, ":/ ")?,
                MusicCommand::Restart => break, // MML では無限ループは書けないらしい
                MusicCommand::End => break,
            }
        }

        Ok(())
    }

    fn note_to_str(note: u8) -> &'static str {
        match note {
            0 => "C",
            1 => "C+",
            2 => "D",
            3 => "D+",
            4 => "E",
            5 => "F",
            6 => "F+",
            7 => "G",
            8 => "G+",
            9 => "A",
            10 => "A+",
            11 => "B",
            _ => unreachable!(),
        }
    }

    fn length_to_tick(length: u32) -> u32 {
        2 * length
    }
}

pub fn load_musics(rom: &Rom) -> Vec<Music> {
    let cfgs = load_music_cfgs(rom);
    let ptrss = load_music_ptrss(rom);

    itertools::zip(cfgs, ptrss)
        .enumerate()
        .map(|(i, ((sq_envelope, sq_duty), ptrs))| {
            let id = u8::try_from(i + 1).unwrap();

            // sq1 トラックは必ず 0xFE または 0xFF で終端されている。
            let (track_sq1, length_sq1) = load_track(rom, ptrs[0], None);

            // ループ曲(sq1 が 0xFE 終端)の場合、sq2, tri トラックには 0xFE 終端がない。
            // よって、load_track() に length_expect 引数を与える必要がある。
            let music_loop = matches!(track_sq1.last().unwrap(), MusicCommand::Restart);
            let length_expect = if music_loop { Some(length_sq1) } else { None };
            let (mut track_sq2, length_sq2) = load_track(rom, ptrs[1], length_expect);
            let (mut track_tri, length_tri) = load_track(rom, ptrs[2], length_expect);
            if music_loop {
                track_sq2.push(MusicCommand::Restart);
                track_tri.push(MusicCommand::Restart);
            }

            assert_eq!(length_sq1, length_sq2);
            assert_eq!(length_sq1, length_tri);

            Music {
                id,
                sq_envelope,
                sq_duty,
                track_sq1,
                track_sq2,
                track_tri,
            }
        })
        .collect()
}

/// (sq_envelope, sq_duty) の配列を返す。
fn load_music_cfgs(rom: &Rom) -> Vec<(u8, SquareDuty)> {
    rom.prg[prg_offset(0xB716)..]
        .iter()
        .take(MUSIC_COUNT)
        .map(|&b| {
            assert_eq!(b & 0x30, 0);
            let sq_envelope = b & 0x0F;
            let sq_duty = SquareDuty::new(b >> 6);
            (sq_envelope, sq_duty)
        })
        .collect()
}

fn load_music_ptrss(rom: &Rom) -> Vec<[u16; 3]> {
    rom.prg[prg_offset(0xBBA6)..]
        .chunks(6)
        .take(MUSIC_COUNT)
        .map(|buf| {
            [
                LE::read_u16(&buf[0..]),
                LE::read_u16(&buf[2..]),
                LE::read_u16(&buf[4..]),
            ]
        })
        .collect()
}

/// rom 内アドレス ptr からトラックを読み込む。
/// length_expect が指定された場合、音長の総和がちょうど length_expect になるまで読み込む。
/// (トラック, 音長の総和) を返す。
///
/// length_expect 引数が必要な理由は、ループ曲(sq1 トラックが 0xFE で終わるもの)の場合、
/// sq2, tri トラックには 0xFE 終端がないため。
fn load_track(rom: &Rom, ptr: u16, length_expect: Option<u32>) -> (Vec<MusicCommand>, u32) {
    let mut track = vec![];

    let prg = &rom.prg;
    let mut offset = 0;

    let mut length: u32 = 0; // トラックの音長の総和
    let mut length_unit: Option<u8> = None; // 現在の単位音長
    let mut loop_count: Option<u8> = None; // 現在のループ回数
    let mut length_loop: u32 = 0; // ループ内の音長の総和

    macro_rules! add_length_unit {
        () => {{
            let length_unit = length_unit.expect("length_unit is not set");
            if loop_count.is_some() {
                length_loop += u32::from(length_unit);
            } else {
                length += u32::from(length_unit);
            }
        }};
    }

    loop {
        let op = prg[prg_offset(ptr + offset)];
        offset += 1;

        match op {
            // 休符
            0 => {
                track.push(MusicCommand::new_rest());
                add_length_unit!();
            }
            // 音符
            25..=0x7F => {
                let value = op - 1;
                let octave = 1 + value / 12;
                let note = value % 12;
                track.push(MusicCommand::new_tone(octave, note));
                add_length_unit!();
            }
            // 音長設定
            0x80..=0xEF => {
                length_unit = Some(op & 0x7F);
                track.push(MusicCommand::new_set_length(length_unit.unwrap()));
            }
            // ループ末尾
            0xFC => {
                let count = loop_count.expect("not in loop");
                track.push(MusicCommand::new_loop_end());
                length += u32::from(count) * length_loop;
                loop_count = None;
                length_loop = 0;
            }
            // ループ開始
            0xFD => {
                assert!(loop_count.is_none(), "nested loop is not permitted");
                loop_count = Some(prg[prg_offset(ptr + offset)]);
                offset += 1;
                track.push(MusicCommand::new_loop_begin(loop_count.unwrap()));
            }
            // 曲の先頭から再開
            0xFE => {
                assert!(loop_count.is_none(), "unclosed loop");
                track.push(MusicCommand::new_restart());
                break;
            }
            // 曲の終了
            0xFF => {
                assert!(loop_count.is_none(), "unclosed loop");
                track.push(MusicCommand::new_end());
                break;
            }
            _ => panic!(
                "invalid track op: ptr={:#06X}, offset={:#06X}, op={:#04X}",
                ptr, offset, op
            ),
        }

        if let Some(length_expect) = length_expect {
            assert!(length <= length_expect);
            if length == length_expect {
                break;
            }
        }
    }

    (track, length)
}
