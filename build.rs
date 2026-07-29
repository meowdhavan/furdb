use std::path::Path;

const PROTO_FILE: &str = "proto/furdb.proto";
const PROTO_DIR: &str = "proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // `prost-build` shells out to `protoc`, which is not part of a stock Rust
    // toolchain. Vendoring it keeps `cargo build` working without any system
    // package being installed first.
    if std::env::var_os("PROTOC").is_none() {
        std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    }

    tonic_prost_build::compile_protos(Path::new(PROTO_FILE))?;

    println!("cargo:rerun-if-changed={PROTO_FILE}");
    println!("cargo:rerun-if-changed={PROTO_DIR}");

    Ok(())
}
