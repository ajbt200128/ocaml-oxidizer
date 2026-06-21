//! `ox update`: download the latest release binary for this platform and replace
//! the running executable with it. Uses `curl` (as install.sh and the action do)
//! so it works regardless of the `networking` feature.

use std::process::Command;

const DEFAULT_REPO: &str = "ajbt200128/ocaml-oxidizer";

/// The release asset name for the host platform, e.g. `ox-macos-aarch64`.
fn asset_name() -> Result<String, String> {
    let os = match std::env::consts::OS {
        "macos" => "macos",
        "linux" => "linux",
        other => return Err(format!("unsupported OS: {other}")),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => return Err(format!("unsupported arch: {other}")),
    };
    Ok(format!("ox-{os}-{arch}"))
}

fn curl(args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("curl")
        .args(args)
        .output()
        .map_err(|e| format!("running curl: {e}"))?;
    if !out.status.success() {
        return Err(format!("curl failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(out.stdout)
}

/// Scrape `"field": "value"` out of a JSON blob — the same heuristic install.sh
/// uses (`grep | cut`), so we avoid a JSON dependency.
fn json_string_field<'a>(body: &'a str, field: &str) -> Option<&'a str> {
    let key = format!("\"{field}\"");
    let after = &body[body.find(&key)? + key.len()..];
    let after = &after[after.find(':')? + 1..];
    let after = &after[after.find('"')? + 1..];
    let end = after.find('"')?;
    Some(&after[..end])
}

fn latest_tag(repo: &str) -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let body = curl(&["-fsSL", &url])?;
    let body = String::from_utf8_lossy(&body);
    json_string_field(&body, "tag_name")
        .map(str::to_string)
        .ok_or_else(|| format!("could not find a latest release for {repo}"))
}

/// Download the latest release for this platform and atomically replace the
/// running binary. Returns a short status message on success.
pub fn run() -> Result<String, String> {
    let repo = std::env::var("OCAML_OXIDIZER_REPO").unwrap_or_else(|_| DEFAULT_REPO.to_string());
    let asset = asset_name()?;
    let tag = latest_tag(&repo)?;

    let current = std::env::current_exe().map_err(|e| format!("finding current exe: {e}"))?;
    // Download alongside the current binary so the final rename stays on one
    // filesystem (and so it lands where we have write access, or fails early).
    let mut tmp = current.clone();
    let mut name = current.file_name().ok_or("current exe has no file name")?.to_os_string();
    name.push(".new");
    tmp.set_file_name(name);

    let url = format!("https://github.com/{repo}/releases/download/{tag}/{asset}");
    println!("downloading {asset} ({tag})");
    if let Err(e) = curl(&["-fsSL", &url, "-o", &tmp.to_string_lossy()]) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755)) {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("chmod {}: {e}", tmp.display()));
        }
    }

    // Renaming over the running binary is fine on Unix: the live process keeps
    // the old inode open until it exits.
    if let Err(e) = std::fs::rename(&tmp, &current) {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("replacing {}: {e}", current.display()));
    }
    Ok(format!("updated ox to {tag}: {}", current.display()))
}
