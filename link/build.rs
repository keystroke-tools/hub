fn main() {
    println!("cargo:rerun-if-changed=schema/host.capnp");

    ::capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/host.capnp")
        .output_path("src/schema")
        .run()
        .expect("Failed to compile Cap'n Proto schema");
}
