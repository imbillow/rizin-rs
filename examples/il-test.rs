use rizin_rs::wrapper::Core;

fn main() -> Result<(), ()> {
    let core = Core::new();
    core.set("analysis.arch", "pic")?;
    core.set("analysis.cpu", "pic18")?;

    let bytes: [u8; 4] = [1, 2, 3, 4];
    let op = core.analysis_op(&bytes, 0)?;
    println!("{} {}", op.mnemonic()?, op.il_str(false)?);
    Ok(())
}
