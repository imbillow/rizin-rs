use std::error::Error;
use std::rc::Rc;

use clap::Parser;
use dashmap::DashMap;
use hex::ToHex;
use itertools::Itertools;
use il_tests::gen;

use rizin_rs::wrapper::{AnalysisOp, Core};

struct Instruction {
    bytes: Vec<u8>,
    mnemonic: Rc<String>,
    op: AnalysisOp,
}

impl Instruction {
    fn from_bytes(core: &Core, bytes: &[u8], addr: usize) -> Result<Self, ()> {
        let op = core.analysis_op(bytes, addr)?;
        let mnemonic = op.mnemonic()?;
        let bytes = &bytes[0..op.0.size as usize];
        match mnemonic.split_whitespace().next() {
            None => Err(()),
            Some(m) => Ok(Self {
                bytes: Vec::from(bytes),
                mnemonic: Rc::new(m.to_string()),
                op,
            }),
        }
    }
}

impl Instruction {
    fn try_to_string(&self) -> rizin_rs::wrapper::Result<String> {
        let op_str = self.op.mnemonic()?;
        let il_str = self.op.il_str(false)?;
        Ok(format!(
            "d \"{}\" {} {:#08x} {}",
            op_str,
            self.bytes.encode_hex::<String>(),
            self.op.0.addr,
            il_str
        ))
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "pic")]
    arch: String,

    #[arg(short, long, default_value = "pic18")]
    cpu: String,

    #[arg(short, long)]
    max: Option<u32>,
}

const INST_LIMIT: usize = 0x8_usize;
const ADDRS: [usize; 2] = [0, 0xff00];

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let max = args.max.unwrap_or(u16::MAX as _);

    let map = DashMap::<Rc<String>, usize>::new();
    let core = {
        let core = Core::new();
        core.set("analysis.arch", &args.arch).unwrap();
        core.set("analysis.cpu", &args.cpu).unwrap();
        core.set("asm.cpu", &args.cpu).unwrap();
        core
    };



    // let _ = (0..max)
    //     .flat_map(|x| {
    //         let b = x.to_le_bytes();
    //         ADDRS
    //             .iter()
    //             .filter_map(|addr| {
    //                 let inst = { Instruction::from_bytes(&core, &b, addr.clone()) };
    //                 if inst.is_err() {
    //                     return None;
    //                 }
    //                 let inst = inst.unwrap();
    //                 let entry = map.get_mut(&inst.mnemonic);
    //                 match entry {
    //                     Some(x) if *x > INST_LIMIT => return None,
    //                     _ => {}
    //                 }
    //
    //                 match entry {
    //                     None => {
    //                         map.insert(inst.mnemonic.clone(), 1);
    //                     }
    //                     Some(mut k) => {
    //                         *k += 1;
    //                     }
    //                 };
    //                 Some(inst)
    //             })
    //             .collect::<Vec<_>>()
    //     })
    //     .sorted_by_key(|x| x.mnemonic.clone())
    //     .for_each(|x| {
    //         let _ = x.try_to_string().map(|str| println!("{}", str));
    //     });
    Ok(())
}
