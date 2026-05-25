fn main() {
    copy_www_to_runtime_dir();
    tauri_build::build()
}

fn copy_www_to_runtime_dir() {
    println!("cargo:rerun-if-changed=../www");

    let manifest_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"),
    );
    let source = manifest_dir.join("..").join("www");

    if !source.is_dir() {
        return;
    }

    let profile = std::env::var("PROFILE").expect("PROFILE is not set");
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is not set"));
    let runtime_dir = out_dir
        .ancestors()
        .find(|path| path.file_name().and_then(|name| name.to_str()) == Some(profile.as_str()))
        .expect("failed to resolve Cargo profile output directory");
    let destination = runtime_dir.join("www");

    if destination.exists() {
        std::fs::remove_dir_all(&destination)
            .expect("failed to remove existing runtime www directory");
    }

    copy_dir_all(&source, &destination).expect("failed to copy www directory next to runtime");
}

fn copy_dir_all(source: &std::path::Path, destination: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(destination)?;

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), target)?;
        }
    }

    Ok(())
}
