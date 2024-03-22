// use std::ffi::c_ulonglong;
// use std::ptr::null_mut;
// use arbitrary::{Arbitrary, Unstructured, Error};
// use crate::*;
//
// pub fn abbrev(input: &[u8]) -> Result<RzBinDwarfAbbrev, ()> {
//     unsafe {
//         let buf = rz_buf_new_with_bytes(input.as_ptr(), input.len() as c_ulonglong);
//         if buf.is_null() {
//             return Err(());
//         }
//         let abbrev = rz_bin _dwarf_abbrev_from_buf(buf);
//         if abbrev.is_null() {
//             return Err(());
//         }
//         Ok(*abbrev)
//     }
// }
//
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
//
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