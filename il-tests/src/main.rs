use std::cmp::min;
use std::error::Error;
use std::fmt::{Display, Formatter};

use chashmap::CHashMap;
use hex::ToHex;

use rizin_rs::wrapper::Core;

struct Instruction {
    addr: usize,
    bytes: Vec<u8>,
    inst: String,
    operands: Option<String>,
    il: String,
}

impl Instruction {
    fn from_bytes(core: &Core, bytes: &[u8], addr: usize) -> Result<Self, ()> {
        let op = core.analysis_op(bytes, addr)?;
        let mnemonic = op.mnemonic()?;
        let ms = mnemonic.split_once(' ');
        Ok(Self {
            addr,
            bytes: Vec::from(bytes),
            inst: ms.map_or(mnemonic.to_string(), |(a, _)| a.to_string()),
            operands: ms.map_or(None, |(_, b)| Some(b.to_string())),
            il: op.il_str(false)?,
        })
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(operands) = &self.operands {
            write!(f, "d \"{} {}\" ", self.inst, operands)?;
        } else {
            write!(f, "d \"{}\" ", self.inst)?;
        }
        write!(
            f,
            "{} {:#08x} {}",
            self.bytes.encode_hex::<String>(),
            self.addr,
            self.il
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    const INST_LIMIT: usize = 0x8_usize;
    const MAX: u32 = u32::MAX;
    let map = CHashMap::<String, usize>::new();
    let n = MAX / (rayon::current_num_threads() as u32);

    let pool = rayon::ThreadPoolBuilder::new().build()?;
    pool.spawn_broadcast(move |ctx| {
        let core = Core::new();
        core.set("analysis.arch", "pic").unwrap();
        core.set("analysis.cpu", "pic18").unwrap();
        let addrs = vec![0, 0xff00];
        let begin: u32 = (ctx.index() as u32) * n;

        for x in begin..min(begin + n, MAX) {
            let b: [u8; 4] = x.to_le_bytes();
            for addr in addrs.clone() {
                let inst = Instruction::from_bytes(&core, &b, addr);
                if inst.is_err() {
                    continue;
                }
                let inst = inst.unwrap();
                match map.get(&inst.inst) {
                    Some(x) if *x > INST_LIMIT => {
                        continue;
                    }
                    _ => {}
                }

                println!("{}", inst);
                match map.get_mut(&inst.inst) {
                    None => {
                        map.insert_new(inst.inst, 1);
                    }
                    Some(mut k) => {
                        *k += 1;
                    }
                }
            }
        }
    });
    pool.broadcast(|_| {});
    Ok(())
}
