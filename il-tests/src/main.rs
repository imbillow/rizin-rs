use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Mutex;

use hex_slice::AsHex;

use rayon::prelude::*;
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
            "{:x} {:#08x} {}",
            self.bytes.plain_hex(false),
            self.addr,
            self.il
        )
    }
}

fn main() -> Result<(), ()> {
    let addrs = vec![0, 0xff00];
    let insts = Mutex::new(HashMap::<String, usize>::new());
    const INST_LIMIT: usize = 0x8_usize;

    let core = Mutex::new(Core::new());
    {
        let co = core.lock().unwrap();
        co.set("analysis.arch", "pic").unwrap();
        co.set("analysis.cpu", "pic18").unwrap();
    }

    (0x1000_u32..u32::MAX).into_par_iter().for_each(|x| {
        let core = core.lock().unwrap();
        let b: [u8; 4] = x.to_le_bytes();
        for addr in addrs.clone() {
            if let Ok(inst) = Instruction::from_bytes(&core, &b, addr) {
                let mut insts = insts.lock().unwrap();
                match insts.get_mut(&inst.inst) {
                    Some(k) if *k > INST_LIMIT => continue,
                    Some(k) => {
                        println!("{}", inst);
                        *k += 1;
                    }
                    None => {
                        println!("{}", inst);
                        insts.insert(inst.inst, 1);
                    }
                }
            }
        }
    });

    Ok(())
}
