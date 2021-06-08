use std::path::PathBuf;

use structopt::StructOpt;

use star_soldier_extract::*;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    path_rom: PathBuf,

    #[structopt(parse(try_from_os_str = parse_directory))]
    dir_out: PathBuf,
}

fn parse_directory(s: &std::ffi::OsStr) -> Result<PathBuf, std::ffi::OsString> {
    let dir = PathBuf::from(s);

    dir.is_dir().then(|| dir).ok_or_else(|| s.to_owned())
}

fn main() -> eyre::Result<()> {
    let opt = Opt::from_args();

    let rom = Rom::from_ines_bytes(std::fs::read(opt.path_rom)?)?;
    let game = Game::from_rom(&rom);

    for &second_round in &[false, true] {
        let imgs = game.meta_sprite_images(second_round);
        for (i, img) in imgs.iter().enumerate() {
            let path_out = opt.dir_out.join(format!(
                "MetaSprite-{}-{:03}.png",
                if second_round { 2 } else { 1 },
                i
            ));
            img.save(path_out)?;
        }
    }

    Ok(())
}
