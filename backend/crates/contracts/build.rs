fn main() {
    let proto_root = "proto";
    let protos = [
        "proto/identity.proto",
        "proto/event_command.proto",
        "proto/event_query.proto",
        "proto/reporting.proto",
    ];

    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc is required");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&protos, &[proto_root])
        .expect("failed to compile service contracts");
}
