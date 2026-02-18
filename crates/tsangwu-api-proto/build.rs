fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = &[
        "../../proto/common.proto",
        "../../proto/user.proto",
        "../../proto/auth.proto",
        "../../proto/project.proto",
        "../../proto/asset.proto",
        "../../proto/generation.proto",
        "../../proto/agent.proto",
        "../../proto/culture.proto",
    ];
    let include_dirs = &["../../proto"];

    for proto in proto_files {
        println!("cargo:rerun-if-changed={}", proto);
    }
    println!("cargo:rerun-if-changed=../../proto");

    // 尝试编译 proto；若 protoc 不可用则生成空桩文件
    match tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(proto_files, include_dirs)
    {
        Ok(_) => {}
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("protoc") || err_msg.contains("NotFound") {
                eprintln!("cargo:warning=protoc 未找到，生成空桩文件。请安装 protoc 以启用 gRPC 代码生成。");
                let out_dir = std::env::var("OUT_DIR")?;
                let packages = [
                    "tsangwu.common",
                    "tsangwu.user",
                    "tsangwu.auth",
                    "tsangwu.project",
                    "tsangwu.asset",
                    "tsangwu.generation",
                    "tsangwu.agent",
                    "tsangwu.culture",
                ];
                for pkg in packages {
                    let path = std::path::Path::new(&out_dir).join(format!("{}.rs", pkg));
                    std::fs::write(&path, "// protoc not available — stub\n")?;
                }
            } else {
                return Err(e.into());
            }
        }
    }

    Ok(())
}
