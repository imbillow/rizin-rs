use std::ffi::CString;
use std::mem::size_of;
use std::path::PathBuf;
use std::ptr;
use std::ptr::null_mut;

use crate::*;

pub struct BinFile {
    bin: *mut RzBin,
    bf: *mut RzBinFile,
}

impl BinFile {
    unsafe fn open(bin: *mut RzBin, path: PathBuf) -> Result<BinFile, ()> {
        let mut rz_bin_opt = RzBinOptions::default();
        rz_bin_options_init(&mut rz_bin_opt, 0, 0, 0, false);
        let cpath = CString::new(path.to_str().unwrap()).unwrap();
        let bf = rz_bin_open(bin, cpath.as_ptr(), &mut rz_bin_opt);
        if bf.is_null() {
            Err(())
        } else {
            Ok(BinFile { bin, bf })
        }
    }
}

impl Drop for BinFile {
    fn drop(&mut self) {
        unsafe {
            rz_bin_file_delete(self.bin, self.bf);
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

pub struct DwarfAbbrev {
    inner: *mut RzBinDwarfAbbrev,
}

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
            Ok(DwarfAbbrev { inner: abbrev })
        }
    }
}

impl Drop for DwarfAbbrev {
    fn drop(&mut self) {
        unsafe {
            rz_bin_dwarf_abbrev_free(self.inner);
        }
    }
}

// pub fn aranges(input: &[u8], big_endian: bool) -> Result<RzBinDwarfARanges, ()> {
//     unsafe {
//         let buf = rz_buf_new_with_bytes(input.as_ptr(), input.len() as c_ulonglong);
//         if buf.is_null() {
//             return Err(());
//         }
//         let x = rz_bin_dwarf_aranges_from_buf(buf, big_endian);
//         if x.is_null() {
//             return Err(());
//         }
//         Ok(*x)
//     }
// }
//
// pub fn info(input: &[u8], big_endian: bool,
//             abbrev: *mut RzBinDwarfAbbrev,
//             str: Option<*mut RzBinDwarfStr>) -> Result<RzBinDwarfInfo, ()> {
//     unsafe {
//         let buf = rz_buf_new_with_bytes(input.as_ptr(), input.len() as c_ulonglong);
//         if buf.is_null() {
//             return Err(());
//         }
//         let x = rz_bin_dwarf_info_from_buf(
//             buf, big_endian, abbrev, str.unwrap_or(null_mut()));
//         if x.is_null() {
//             return Err(());
//         }
//         Ok(*x)
//     }
// }
//
// pub fn line(input: &[u8],
//             big_endian: bool,
//             encoding: *mut RzBinDwarfEncoding,
//             info: *mut RzBinDwarfInfo)
//             -> Result<RzBinDwarfLineInfo, ()> {
//     unsafe {
//         let buf = rz_buf_new_with_bytes(input.as_ptr(), input.len() as c_ulonglong);
//         if buf.is_null() {
//             return Err(());
//         }
//         let R = RzBinEndianReader{
//             buffer:buf,,
//             big_endian,
//             section_name:,
//             relocations: ,
//         }
//         let x = rz_bin_dwarf_line_new(
//             buf, encoding, info, mask);
//         if x.is_null() {
//             return Err(());
//         }
//         Ok(*x)
//     }
// }

// #[derive(Debug, Default, Clone)]
// pub struct InfoInput {
//     pub info: Vec<u8>,
//     pub abbrev: Vec<u8>,
// }
//
// impl<'a> Arbitrary<'a> for InfoInput {
//     fn arbitrary(raw: &mut Unstructured<'a>) -> Result<Self, Error> {
//         let mut input = Self {
//             info: Vec::with_capacity(raw.arbitrary_len::<u16>()?),
//             abbrev: Vec::with_capacity(raw.arbitrary_len::<u16>()?),
//         };
//         let _ = raw.fill_buffer(input.info.as_mut())?;
//         let _ = raw.fill_buffer(input.abbrev.as_mut())?;
//         Ok(input)
//     }
// }
//
// #[derive(Debug, Default, Clone)]
// pub struct LineInput {
//     pub info: InfoInput,
//     pub line: Vec<u8>,
//     pub encoding: RzBinDwarfEncoding,
// }
//
// impl<'a> Arbitrary<'a> for RzBinDwarfEncoding {
//     fn arbitrary(raw: &mut Unstructured<'a>) -> Result<Self, Error> {
//         Ok(Self {
//             address_size: raw.arbitrary()?,
//             version: raw.arbitrary()?,
//             is_64bit: raw.arbitrary()?,
//         })
//     }
// }
//
// impl<'a> Arbitrary<'a> for LineInput {
//     fn arbitrary(raw: &mut Unstructured<'a>) -> Result<Self, Error> {
//         let mut input = Self {
//             info: InfoInput::arbitrary(raw)?,
//             line: Vec::with_capacity(raw.arbitrary_len::<u16>()?),
//             encoding: raw.arbitrary()?,
//         };
//         let _ = raw.fill_buffer(&mut input.line)?;
//         Ok(input)
//     }
// }
