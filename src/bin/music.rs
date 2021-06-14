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

    for music in load_musics(&rom) {
        let path_out = opt.dir_out.join(format!("music-{:02}.mml", music.id));
        music.write_mml(std::fs::File::create(path_out)?)?;
    }

    Ok(())
}
