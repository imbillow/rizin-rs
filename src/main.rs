use std::error::Error;
use std::ffi::*;
use rizin_rs::*;

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        // let rz_bin = rz_bin_new();
        // let io = rz_io_new();
        // rz_io_bind(io, &mut (*rz_bin).iob);
        // let mut rz_bin_opt = RzBinOptions::default();
        // rz_bin_options_init(&mut rz_bin_opt, 0, 0, 0, false);
        //
        // let path = CString::new("target/debug/librizin_rs.rlib")?;
        // rz_bin_open(rz_bin, path.as_ptr(), &mut rz_bin_opt);
        //
        // let dw = rz_bin_dwarf_from_file((*rz_bin).cur);
        // println!("{:#?} {}", dw, (*(*dw).info).units.len);
    }
    println!("Hello, world!");
    Ok(())
}
