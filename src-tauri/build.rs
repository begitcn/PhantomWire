use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest_path = PathBuf::from("admin.manifest");
        if manifest_path.exists() {
            let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is required"));
            let rc_path = out_dir.join("admin-manifest.rc");
            let manifest_abs = manifest_path
                .canonicalize()
                .expect("failed to resolve admin.manifest path")
                .to_string_lossy()
                .replace('\\', "\\\\");
            let rc_content = format!("1 RT_MANIFEST \"{}\"\n", manifest_abs);
            fs::write(&rc_path, rc_content).expect("failed to write manifest rc file");
            let _ = embed_resource::compile(rc_path, embed_resource::NONE);
        }
    }
    tauri_build::build()
}
