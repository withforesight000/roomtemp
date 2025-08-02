use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .file_descriptor_set_path(
            PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set")).join("tempgrpcd.bin"),
        )
        .out_dir("src/pb")
        .compile_protos(&["src/pb/tempgrpcd.proto"], &["src/"])?;
    Ok(())
}
