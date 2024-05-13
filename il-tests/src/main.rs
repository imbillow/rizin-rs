use std::cmp::min;
use std::error::Error;
use std::fmt::Display;
use std::sync::{Arc, Mutex};

use clap::Parser;
use dashmap::DashMap;
use hex::ToHex;

use rizin_rs::wrapper::{AnalysisOp, Core};

struct Instruction {
    bytes: Vec<u8>,
    mnemonic: String,
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
                mnemonic: m.to_string(),
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

    let n = max / (rayon::current_num_threads() as u32);
    let map = Arc::new(DashMap::<String, usize>::new());
    let core = Arc::new({
        let core = Core::new();
        core.set("analysis.arch", &args.arch).unwrap();
        core.set("analysis.cpu", &args.cpu).unwrap();
        core.set("asm.cpu", &args.cpu).unwrap();
        Mutex::new(core)
    });
    let runner =
        |core: Arc<Mutex<Core>>, map: Arc<DashMap<String, usize>>, x: u32| -> Result<(), _> {
            let b: [u8; 4] = x.to_le_bytes();
            for addr in ADDRS {
                let inst = {
                    let core = core.lock().map_err(|_| ())?;
                    Instruction::from_bytes(&core, &b, addr)
                };
                if inst.is_err() {
                    continue;
                }
                let inst = inst.unwrap();
                let entry = map.get_mut(&inst.mnemonic);
                match entry {
                    Some(x) if *x > INST_LIMIT => {
                        continue;
                    }
                    _ => {}
                }

                if let Ok(inst_str) = inst.try_to_string() {
                    println!("{}", inst_str);
                    match entry {
                        None => {
                            map.insert(inst.mnemonic, 1);
                        }
                        Some(mut k) => {
                            *k += 1;
                        }
                    }
                }
            }
            Ok::<(), ()>(())
        };

    let pool = rayon::ThreadPoolBuilder::new().build()?;
    pool.spawn_broadcast(move |ctx| {
        let begin: u32 = (ctx.index() as u32) * n;
        for x in begin..min(begin + n, max) {
            let _ = runner(core.clone(), map.clone(), x);
        }
    });
    pool.broadcast(|_| {});
    Ok(())
}
