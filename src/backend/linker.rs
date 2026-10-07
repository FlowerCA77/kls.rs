use std::path::PathBuf;
use std::process::Command;

use crate::Result;

fn find_stdlib_path() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("KLS_STDLIB_PATH") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Ok(path);
        }
    }

    let candidates = [
        "kls-stdlib/target/release/libkls_stdlib.a",
        "kls-stdlib/target/debug/libkls_stdlib.a",
        "target/release/libkls_stdlib.a",
        "target/debug/libkls_stdlib.a",
    ];

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    for candidate in candidates {
        let path = PathBuf::from(manifest_dir).join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    Err("cannot find libkls_stdlib.a".into())
}

pub fn link(objects: &[PathBuf], output: &PathBuf) -> Result<()> {
    let mut cmd = Command::new(std::env::var("CC").unwrap_or_else(|_| "cc".into()));
    for obj in objects {
        cmd.arg(obj);
    }
    cmd.arg("-o").arg(output);

    let stdlib_path = find_stdlib_path()?;
    cmd.arg(&stdlib_path);

    cmd.arg("-lpthread");
    cmd.arg("-lm");
    #[cfg(target_os = "macos")]
    {
        cmd.arg("-framework").arg("Security");
        cmd.arg("-framework").arg("CoreFoundation");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("linking failed: {:?}", status).into());
    }
    Ok(())
}
