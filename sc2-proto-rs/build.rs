#[cfg(feature = "protoc-rust")]
use protoc_rust::{Codegen, Customize};
#[cfg(feature = "protoc-rust")]
use std::{env, ffi::OsStr, fs, path::Path};

#[cfg(feature = "protoc-rust")]
fn proto_modules(proto_dir: &Path) -> Result<Vec<String>, String> {
    let rd = fs::read_dir(proto_dir).map_err(|e| {
        format!(
            "Could not read protobuf directory '{}': {}",
            proto_dir.display(),
            e
        )
    })?;

    let mods = rd
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() && path.extension() == Some(OsStr::new("proto")) {
                path.file_stem()
                    .and_then(|n| n.to_os_string().into_string().ok())
            } else {
                None
            }
        })
        .collect();

    Ok(mods)
}

#[cfg(feature = "protoc-rust")]
fn main() {
    let in_dir = "./s2client-proto/s2clientprotocol";
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| String::from("OUT_DIR_NOT_SET"));

    // Read list of all input protobuf files
    let input_mods = match proto_modules(Path::new(in_dir)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("build script: {}", e);
            eprintln!(
                "Hint: the repository does not contain the 's2client-proto' checkout. Falling back to pre-generated sources in 'src/'."
            );

            // Attempt to copy the pre-generated sources from `src/` into OUT_DIR and write a lib.rs
            let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| String::from("OUT_DIR_NOT_SET"));
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| String::from("."));
            let fallback = [
                "common", "data", "debug", "error", "query", "raw", "sc2api", "score", "spatial",
                "ui",
            ];

            for m in &fallback {
                let src = format!("{}/src/{}.rs", manifest_dir, m);
                let dst = format!("{}/{}.rs", out_dir, m);
                if let Err(e) = fs::copy(&src, &dst) {
                    eprintln!("Failed to copy '{}' to '{}': {}", src, dst, e);
                    std::process::exit(1);
                }
            }

            let content = fallback
                .iter()
                .map(|s| format!("pub mod {};", s))
                .collect::<Vec<_>>()
                .join("\n");

            if let Err(e) = fs::write(format!("{}/{}", out_dir, "lib.rs"), content) {
                eprintln!(
                    "Failed to write fallback lib.rs in OUT_DIR '{}': {}",
                    out_dir, e
                );
                std::process::exit(1);
            }

            return;
        }
    };

    let input_files: Vec<String> = input_mods
        .iter()
        .map(|s| format!("{}/{}.proto", in_dir, s))
        .collect();

    // Compile protocol buffers
    if let Err(e) = Codegen::new()
        .out_dir(&out_dir)
        .include("s2client-proto/")
        .inputs(input_files)
        .customize(Customize {
            expose_fields: Some(true),
            ..Default::default()
        })
        .run()
    {
        eprintln!("protoc-rust failed: {:#?}", e);
        std::process::exit(1);
    }
    println!("protobufs were generated successfully");

    // Generate the lib.rs source code
    if let Err(e) = fs::write(
        format!("{}/{}", out_dir, "lib.rs"),
        input_mods
            .iter()
            .map(|s| format!("pub mod {};", s))
            .collect::<Vec<_>>()
            .join("\n"),
    ) {
        eprintln!("Failed to write lib.rs in OUT_DIR '{}': {}", out_dir, e);
        std::process::exit(1);
    }
}

#[cfg(not(feature = "protoc-rust"))]
fn main() {
    println!("using pre-generated *.rs files in 'src/'");
}
