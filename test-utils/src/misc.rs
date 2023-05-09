use std::{fs, path::Path};

use ckb_types::{packed, prelude::*};
use rand::{thread_rng, Rng as _};

pub fn load_data_from_file(data_dir: &Path, filename: &str) -> Vec<u8> {
    let path = data_dir.join(filename);
    fs::read(path).unwrap()
}

pub fn load_contract_from_file(path_str: &str) -> Vec<u8> {
    fs::read(path_str).unwrap()
}

pub fn randomize_hash() -> packed::Byte32 {
    let mut rng = thread_rng();
    let mut buf = [0u8; 32];
    rng.fill(&mut buf);
    buf.pack()
}

pub fn randomize_out_point() -> packed::OutPoint {
    let index: u32 = thread_rng().gen_range(1..100);
    packed::OutPoint::new_builder()
        .tx_hash(randomize_hash())
        .index(index.pack())
        .build()
}

pub fn randomize_bytes() -> Vec<u8> {
    let mut rng = thread_rng();
    let len: usize = rng.gen_range(0..64);
    let mut buf = vec![0u8; len];
    rng.fill(&mut buf[..]);
    buf
}

pub fn randomize_fixed_bytes<const N: usize>() -> [u8; N] {
    let mut rng = thread_rng();
    let mut buf = [0u8; N];
    rng.fill(&mut buf[..]);
    buf
}
