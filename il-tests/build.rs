use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::anyhow;
use sleigh2rust::generate_disassembler;

fn main() {
    let p = PathBuf::from(format!(
        "ghidra/Ghidra/Processors/{}/data/languages/{}.slaspec",
        "tricore", "tricore"
    ));
    let p = p
        .canonicalize()
        .map_err(|e| anyhow!("{} {:?} {:?}", e, env::current_dir(), &p))
        .unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let x = generate_disassembler(&p).unwrap();
    let f = File::create(out_dir.join("tricore.rs")).unwrap();
    let mut w = BufWriter::new(f);
    w.write(x.to_string().as_bytes()).unwrap();
}
