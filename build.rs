use std::io::Result;

#[cfg(any(feature = "rust", feature = "flutter"))]
fn main() -> Result<()> {
    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    std::env::set_var("PROTOC", protoc);
    prost_build::compile_protos(&["esf.proto"], &["."])?;
    Ok(())
}

#[cfg(not(any(feature = "rust", feature = "flutter")))]
fn main() -> Result<()> {
    Ok(())
}
