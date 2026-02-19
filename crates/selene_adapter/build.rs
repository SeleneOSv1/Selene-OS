fn main() {
    let proto_file = "proto/voice_ingress.proto";
    println!("cargo:rerun-if-changed={proto_file}");
    tonic_build::configure()
        .build_server(true)
        .compile_protos(&[proto_file], &["proto"])
        .expect("voice ingress proto compilation must succeed");
}
