use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Display;
use std::mem::size_of;
use std::path::PathBuf;
use std::ptr::{addr_of, addr_of_mut, null_mut};

use crate::*;

pub struct Core(pub Box<RzCore>);

impl Drop for Core {
    fn drop(&mut self) {
        let ptr = Box::into_raw(self.0.clone());
        unsafe {
            rz_core_free(ptr);
        }
    }
}
pub struct AnalysisOp(pub RzAnalysisOp);

impl Drop for AnalysisOp {
    fn drop(&mut self) {
        unsafe {
            rz_analysis_op_fini(addr_of_mut!(self.0));
        }
    }
}

pub struct StrBuf(RzStrBuf);

impl StrBuf {
    pub fn new() -> Self {
        let mut sb = RzStrBuf::default();
        unsafe { rz_strbuf_init(addr_of_mut!(sb)) };
        Self(sb)
    }
}

impl Display for StrBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cptr = unsafe { rz_strbuf_drain_nofree(addr_of!(self.0) as _) };
        if cptr.is_null() {
            Ok(())
        } else {
            let cstr = unsafe { CStr::from_ptr(cptr) };
            match cstr.to_str() {
                Ok(str) => f.write_str(str),
                Err(_) => Ok(()),
            }
        }
    }
}

impl Drop for StrBuf {
    fn drop(&mut self) {
        unsafe {
            rz_strbuf_fini(addr_of_mut!(self.0));
        }
    }
}

impl AnalysisOp {
    pub fn mnemonic(&self) -> Result<&str, ()> {
        if self.0.mnemonic.is_null() {
            Err(())
        } else {
            let cstr = unsafe { CStr::from_ptr(self.0.mnemonic) };
            cstr.to_str().map_err(|_| ())
        }
    }

    pub fn il_str(&self, pretty: bool) -> Result<String, ()> {
        if self.0.il_op.is_null() {
            Err(())
        } else {
            let mut sb = StrBuf::new();
            unsafe {
                rz_il_op_effect_stringify(self.0.il_op, addr_of_mut!(sb.0), pretty);
            }
            Ok(sb.to_string())
        }
    }
}

impl Core {
    pub fn new() -> Self {
        let core = unsafe { rz_core_new() };
        if core.is_null() {
            panic!("memory");
        }
        unsafe { Self(Box::from_raw(core)) }
    }

    pub fn analysis_op(&self, bytes: &[u8], addr: usize) -> Result<AnalysisOp, ()> {
        let mut op: AnalysisOp = AnalysisOp(Default::default());
        let res = unsafe {
            rz_analysis_op(
                self.0.analysis,
                addr_of_mut!(op.0),
                addr as _,
                bytes.as_ptr() as _,
                bytes.len() as _,
                RzAnalysisOpMask_RZ_ANALYSIS_OP_MASK_DISASM
                    | RzAnalysisOpMask_RZ_ANALYSIS_OP_MASK_IL,
            )
        };
        if res <= 0 {
            Err(())
        } else {
            Ok(op)
        }
    }

    pub fn set(&self, k: &str, v: &str) -> Result<(), ()> {
        let node = unsafe {
            rz_config_set(
                self.0.config,
                CString::new(k).map_err(|_| ())?.as_ptr(),
                CString::new(v).map_err(|_| ())?.as_ptr(),
            )
        };
        if node.is_null() {
            Err(())
        } else {
            Ok(())
        }
    }
}

pub struct BinFile<'a> {
    core: &'a Core,
    pub bf: *mut RzBinFile,
}

impl Core {
    unsafe fn open(&mut self, path: PathBuf) -> Result<BinFile, ()> {
        let mut rz_bin_opt = RzBinOptions::default();
        rz_bin_options_init(&mut rz_bin_opt, 0, 0, 0, false);
        let cpath = CString::new(path.to_str().unwrap()).unwrap();
        let bf = rz_bin_open(self.0.bin, cpath.as_ptr(), &mut rz_bin_opt);
        if bf.is_null() {
            Err(())
        } else {
            Ok(BinFile { core: self, bf })
        }
    }
}

impl Drop for BinFile<'_> {
    fn drop(&mut self) {
        unsafe {
            rz_bin_file_delete(self.core.0.bin, self.bf);
        }
    }
}

impl RzBinEndianReader {
    fn new(input: &[u8], big_endian: bool) -> Self {
        Self {
            data: input.as_ptr() as _,
            owned: false,
            length: input.len() as _,
            offset: 0,
            big_endian,
            relocations: null_mut(),
        }
    }
}

pub struct DwarfAbbrev(pub *mut RzBinDwarfAbbrev);

impl DwarfAbbrev {
    pub fn new(input: &[u8]) -> Result<DwarfAbbrev, ()> {
        unsafe {
            let R = RzBinEndianReader::new(input, false);
            let Rc = rz_mem_alloc(size_of::<RzBinEndianReader>());
            memcpy(Rc, addr_of!(R) as _, size_of::<RzBinEndianReader>() as _);
            let abbrev = rz_bin_dwarf_abbrev_new(Rc as _);
            if abbrev.is_null() {
                return Err(());
            }
            Ok(DwarfAbbrev(abbrev))
        }
    }
}

impl Drop for DwarfAbbrev {
    fn drop(&mut self) {
        unsafe {
            rz_bin_dwarf_abbrev_free(self.0);
        }
    }
}
