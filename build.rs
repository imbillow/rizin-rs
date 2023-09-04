use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), String> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_dir = out_dir.join("build");
    meson::build("rizin", build_dir.to_str().unwrap());
    let status = Command::new("meson")
        .current_dir(build_dir).args(["install", "--destdir", out_dir.to_str().unwrap()])
        .status().expect("cannot run command");
    if !status.success() {
        return Err("meson install failed".to_string());
    }

    println!("cargo:rustc-link-lib=rz_bin");
    println!("cargo:rustc-link-search=native={}",
             out_dir.join("usr").join("local").join("lib").to_str().unwrap());
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args([
            "-I", out_dir.join("usr").join("local").join("include").to_str().unwrap(),
            "-I", out_dir.join("usr").join("local").join("include").join("librz").to_str().unwrap(),
            "-I", out_dir.join("usr").join("local").join("include").join("librz").join("sdb").to_str().unwrap(),
            "-I", "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include/"
        ])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

// Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    Ok(())
}
