fn main() {
    // 只有在 Windows 平台构建时才嵌入清单
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let _ = embed_resource::compile("admin.manifest", embed_resource::NONE);
    }
    tauri_build::build()
}
