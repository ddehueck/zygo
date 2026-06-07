use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Resolve the proto path relative to this crate's manifest.
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_dir = manifest_dir.join("proto");
    let proto_path = proto_dir.join("orchestrator.proto");

    // Emit a file descriptor set so the `grpc` module can enable gRPC
    // reflection in development builds.
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("orchestrator_descriptor.bin");

    tonic_prost_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .build_client(true)
        .build_server(true)
        .compile_protos(&[proto_path], &[proto_dir])?;

    Ok(())
}
