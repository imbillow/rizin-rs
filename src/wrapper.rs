use std::ffi::CString;
use std::mem::size_of;
use std::path::PathBuf;
use std::ptr;
use std::ptr::null_mut;

use crate::*;

pub struct Core(pub Box<RzCore>);

impl Core {
    pub fn new() -> Self {
        unsafe { Self(Box::from_raw(rz_core_new())) }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        let ptr = Box::into_raw(self.0.clone());
        unsafe {
            rz_core_free(ptr);
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
            memcpy(
                Rc,
                ptr::addr_of!(R) as _,
                size_of::<RzBinEndianReader>() as _,
            );
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
