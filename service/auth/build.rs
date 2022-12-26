use std::path::Path;

fn main() {
    let proto = "proto/token.proto";
    let proto_path: &Path = proto.as_ref();
    let proto_dir = proto_path
        .parent()
        .expect("proto file should reside in a directory");

    tonic_build::compile_protos(proto).unwrap();

    // prevent needing to rebuild if files (or deps) haven't changed
    println!("cargo:rerun-if-changed={}", proto_dir.to_str().unwrap());
}
