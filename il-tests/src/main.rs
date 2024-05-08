use std::cmp::min;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};

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

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let op_str = self.op.mnemonic().map_err(|_| fmt::Error)?;
        let il_str = self.op.il_str(false).map_err(|_| fmt::Error)?;
        write!(
            f,
            "d \"{}\" {} {:#08x} {}",
            op_str,
            self.bytes.encode_hex::<String>(),
            self.op.0.addr,
            il_str
        )
    }
}

const INST_LIMIT: usize = 0x8_usize;
const MAX: u32 = u16::MAX as _;
const ADDRS: [usize; 2] = [0, 0xff00];

fn main() -> Result<(), Box<dyn Error>> {
    let n = MAX / (rayon::current_num_threads() as u32);
    let map = Arc::new(DashMap::<String, usize>::new());
    let core = Arc::new(Mutex::new({
        let core = Core::new();
        core.set("analysis.arch", "pic").unwrap();
        core.set("analysis.cpu", "pic18").unwrap();
        core
    }));
    let runner = |core: Arc<Mutex<Core>>, map: Arc<DashMap<String, usize>>, x: u32| {
        let b: [u8; 4] = x.to_le_bytes();
        for addr in ADDRS {
            let inst = {
                let core = core.lock().unwrap();
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

            println!("{}", inst);
            match entry {
                None => {
                    map.insert(inst.mnemonic, 1);
                }
                Some(mut k) => {
                    *k += 1;
                }
            }
        }
    };

    let pool = rayon::ThreadPoolBuilder::new().build()?;
    pool.spawn_broadcast(move |ctx| {
        let begin: u32 = (ctx.index() as u32) * n;
        for x in begin..min(begin + n, MAX) {
            runner(core.clone(), map.clone(), x);
        }
    });
    pool.broadcast(|_| {});
    Ok(())
}
