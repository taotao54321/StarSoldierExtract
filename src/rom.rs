use std::convert::TryInto;

use eyre::ensure;

#[derive(Debug)]
pub struct Rom {
    pub prg: [u8; 0x8000],
    pub chr: [u8; 0x8000],
}

impl Rom {
    pub fn from_ines_bytes(buf: impl AsRef<[u8]>) -> eyre::Result<Self> {
        const INES_MAGIC: &[u8] = b"NES\x1A";

        let buf = buf.as_ref();
        ensure!(buf.len() == 16 + 0x8000 + 0x8000, "size mismatch");
        ensure!(buf.starts_with(INES_MAGIC), "iNES magic not found");

        let prg = buf[16..][..0x8000].try_into().unwrap();
        let chr = buf[16 + 0x8000..][..0x8000].try_into().unwrap();

        Ok(Self { prg, chr })
    }
}

pub fn prg_offset(addr: u16) -> usize {
    assert!(
        (0x8000..=0xFFFF).contains(&addr),
        "not PRG address: 0x{:04X}",
        addr
    );

    (addr - 0x8000) as usize
}
