use std::path::Path;
use std::process::Command;

use crate::Error;

/// Attempt a git sparse checkout of a single folder.
///
/// `folder` is the top-level directory name in the repo. Clones into a temp
/// directory, sets sparse-checkout to that folder, then moves it into place.
///
/// Tries the versioned tag (`v{version}`) first; falls back to the default
/// branch so the crate works before a release is tagged.
pub(crate) fn try_git_sparse_checkout(
    root: &Path,
    folder: &str,
    version: &str,
    repo_url: &str,
) -> Result<(), Error> {
    if !has_git() {
        return Err(Error::NetworkUnavailable {
            dataset: folder.to_string(),
        });
    }

    let tmp_dir = temp_dir_path(root);

    // Try the versioned tag first, then fall back to default branch.
    let tag = format!("v{version}");
    let cloned = try_git_clone(repo_url, &tmp_dir, Some(&tag))
        .or_else(|_| {
            eprintln!(
                "codec-corpus: warning: tag '{tag}' not found, falling back to default branch. \
                 Data may not match crate version {version}."
            );
            try_git_clone(repo_url, &tmp_dir, None)
        });

    if cloned.is_err() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(Error::NetworkUnavailable {
            dataset: folder.to_string(),
        });
    }

    let sparse_result = Command::new("git")
        .args(["sparse-checkout", "set", &format!("{folder}/")])
        .current_dir(&tmp_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status();

    if !matches!(sparse_result, Ok(s) if s.success()) {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(Error::NetworkUnavailable {
            dataset: folder.to_string(),
        });
    }

    let src = tmp_dir.join(folder);
    let dst = root.join(folder);
    if src.is_dir() {
        std::fs::rename(&src, &dst).map_err(Error::Io)?;
    } else {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(Error::NetworkUnavailable {
            dataset: folder.to_string(),
        });
    }

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

/// Clone with `--depth=1 --filter=blob:none --sparse`. If `branch` is
/// `Some`, adds `--branch <branch>`.
fn try_git_clone(repo_url: &str, dest: &Path, branch: Option<&str>) -> Result<(), Error> {
    let _ = std::fs::remove_dir_all(dest);
    let mut cmd = Command::new("git");
    cmd.args(["clone", "--depth=1", "--filter=blob:none", "--sparse"]);
    if let Some(b) = branch {
        cmd.args(["--branch", b]);
    }
    cmd.arg(repo_url)
        .arg(dest)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());
    let status = cmd.status().map_err(Error::Io)?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::NetworkUnavailable {
            dataset: String::new(),
        })
    }
}

/// Attempt to download the root-folder tarball via HTTP using curl, wget, or
/// powershell, then extract it with the system `tar` command.
///
/// `folder` is the top-level directory name. The tarball is always named
/// `{folder}.tar.gz` in the release assets.
pub(crate) fn try_http_download(
    root: &Path,
    folder: &str,
    version: &str,
) -> Result<(), Error> {
    let url = format!(
        "https://github.com/imazen/codec-corpus/releases/download/v{version}/{folder}.tar.gz"
    );

    let tmp_tar = temp_tar_path(root);
    let downloaded = try_curl(&url, &tmp_tar)
        .or_else(|_| try_wget(&url, &tmp_tar))
        .or_else(|_| try_powershell(&url, &tmp_tar));

    if downloaded.is_err() {
        let _ = std::fs::remove_file(&tmp_tar);
        return Err(Error::NetworkUnavailable {
            dataset: folder.to_string(),
        });
    }

    let tmp_extract = temp_dir_path(root);
    std::fs::create_dir_all(&tmp_extract).map_err(Error::Io)?;

    let tar_status = Command::new("tar")
        .args(["xzf"])
        .arg(&tmp_tar)
        .arg("-C")
        .arg(&tmp_extract)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status();

    let _ = std::fs::remove_file(&tmp_tar);

    if !matches!(tar_status, Ok(s) if s.success()) {
        let _ = std::fs::remove_dir_all(&tmp_extract);
        return Err(Error::Io(std::io::Error::other(
            "tar extraction failed",
        )));
    }

    let src = tmp_extract.join(folder);
    let dst = root.join(folder);
    if src.is_dir() {
        std::fs::rename(&src, &dst).map_err(Error::Io)?;
    } else {
        let _ = std::fs::remove_dir_all(&tmp_extract);
        return Err(Error::NetworkUnavailable {
            dataset: folder.to_string(),
        });
    }

    let _ = std::fs::remove_dir_all(&tmp_extract);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tool detection helpers
// ---------------------------------------------------------------------------

fn has_git() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn try_curl(url: &str, dest: &Path) -> Result<(), Error> {
    let status = Command::new("curl")
        .args(["-fSL", "-o"])
        .arg(dest)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(Error::Io)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::NetworkUnavailable {
            dataset: String::new(),
        })
    }
}

fn try_wget(url: &str, dest: &Path) -> Result<(), Error> {
    let status = Command::new("wget")
        .args(["-q", "-O"])
        .arg(dest)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(Error::Io)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::NetworkUnavailable {
            dataset: String::new(),
        })
    }
}

fn try_powershell(url: &str, dest: &Path) -> Result<(), Error> {
    // Use a parameterized script block to avoid command injection via
    // single-quote breakout in the URL or file path.
    let script = "param($u,$o) Invoke-WebRequest -Uri $u -OutFile $o";
    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", script, url])
        .arg(dest)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(Error::Io)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::NetworkUnavailable {
            dataset: String::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Temp path helpers
// ---------------------------------------------------------------------------

fn temp_dir_path(root: &Path) -> std::path::PathBuf {
    let pid = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    root.join(format!(".tmp-{pid}-{ts}"))
}

fn temp_tar_path(root: &Path) -> std::path::PathBuf {
    let pid = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    root.join(format!(".tmp-{pid}-{ts}.tar.gz"))
}
