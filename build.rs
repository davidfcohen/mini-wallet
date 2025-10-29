use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let descriptor_path = PathBuf::from(out_dir).join("descriptor.bin");

    tonic_prost_build::configure()
        .build_client(false)
        .file_descriptor_set_path(descriptor_path)
        .compile_protos(protos(), &["proto"])?;
    Ok(())
}

const fn protos() -> &'static [&'static str] {
    &["proto/wallet.proto"]
}
