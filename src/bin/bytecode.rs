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

    for enemy_group in load_enemy_groups(&rom) {
        if let Some(bytecode) = enemy_group.bytecode {
            let path_out = opt
                .dir_out
                .join(format!("bytecode-{:02}.bin", enemy_group.id));
            std::fs::write(path_out, bytecode)?;
        }
    }

    Ok(())
}
