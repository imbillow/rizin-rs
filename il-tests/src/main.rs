#![feature(coroutine_trait)]
#![feature(coroutines)]

use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use bitvec::prelude::*;
use clap::Parser;
use hex::ToHex;
use itertools::Itertools;
use rand::Rng;

use rizin_rs::wrapper::{AnalysisOp, Core};
use sleigh_rs::file_to_sleigh;
use sleigh_rs::pattern::BitConstraint;

struct Instruction {
    bytes: Vec<u8>,
    mnemonic: Rc<String>,
    op: AnalysisOp,
}

impl Instruction {
    fn from_bytes(core: &Core, bytes: &[u8], addr: usize) -> Result<Self> {
        let op = core.analysis_op(bytes, addr)?;
        let mnemonic = op.mnemonic()?;
        let bytes = &bytes[0..op.0.size as usize];
        match mnemonic.split_whitespace().next() {
            None => Err(anyhow!("Invalid")),
            Some(m) => Ok(Self {
                bytes: Vec::from(bytes),
                mnemonic: Rc::new(m.to_string()),
                op,
            }),
        }
    }
}

impl Instruction {
    fn try_to_string(&self, il: bool) -> rizin_rs::wrapper::Result<String> {
        let mut res = String::new();
        let op_str = self.op.mnemonic()?;
        fmt::write(
            &mut res,
            format_args!(
                "d \"{}\" {} {:#08x}",
                op_str,
                self.bytes.encode_hex::<String>(),
                self.op.0.addr
            ),
        )?;

        if il {
            let il_str = self.op.il_str(false)?;
            fmt::write(&mut res, format_args!(" {}", il_str))?;
        }
        Ok(res)
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "tricore")]
    arch: String,

    #[arg(short, long, default_value = "tricore")]
    cpu: String,
}

struct InstructionConstraint(Vec<BitConstraint>);

impl InstructionConstraint {
    fn sample_data<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec<u8> {
        let iter = self.0.iter().map(|x| match x {
            BitConstraint::Unrestrained => rng.gen_bool(0.5),
            BitConstraint::Defined(x) => *x,
            BitConstraint::Restrained => false,
        });
        let data = BitVec::<u8>::from_iter(iter).as_raw_slice().into();
        data
    }

    fn sample_ops<'a, R: Rng + ?Sized>(
        &'a self,
        core: &'a Core,
        addr: usize,
        rng: &'a mut R,
        count: usize,
    ) -> Vec<Instruction> {
        (0..count)
            .filter_map(|_| {
                let data = self.sample_data(rng);
                Instruction::from_bytes(core, &data, addr).ok()
            })
            .filter(|x| !x.mnemonic.to_lowercase().contains("invalid"))
            .collect_vec()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let core = {
        let core = Core::new();
        core.set("analysis.arch", &args.arch).unwrap();
        core.set("analysis.cpu", &args.cpu).unwrap();
        core.set("asm.cpu", &args.cpu).unwrap();
        core
    };

    let p = PathBuf::from(format!(
        "ghidra/Ghidra/Processors/{}/data/languages/{}.slaspec",
        &args.arch, &args.cpu
    ));
    let sleigh = file_to_sleigh(&p)?;
    let insttbl = sleigh.table(sleigh.instruction_table());
    let mut rng = rand::thread_rng();
    insttbl
        .constructors()
        .iter()
        .flat_map(|ctor| {
            // let m = ctor.display.mneumonic.as_ref().unwrap();
            ctor.pattern
                .pattern_bits_variants(&sleigh)
                .flat_map(|(_, _, b)| {
                    let x = InstructionConstraint(b);
                    x.sample_ops(&core, 0x0, &mut rng, 3)
                })
                .collect_vec()
        })
        .sorted_by_key(|x| x.mnemonic.clone())
        .for_each(|x| {
            let _ = x.try_to_string(true).map(|str| println!("{}", str));
        });
    Ok(())
}
