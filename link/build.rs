fn main() {
    println!("cargo:rerun-if-changed=schema/host.capnp");

    ::capnpc::CompilerCommand::new()
        .src_prefix("schema/shared")
        .file("schema/shared/store.capnp")
        .file("schema/shared/entry.capnp")
        .output_path("src/schema")
        .run()
        .expect("Failed to compile Cap'n Proto schema");
}
