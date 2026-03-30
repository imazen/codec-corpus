use std::path::{Path, PathBuf};
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
    let cloned = try_git_clone(repo_url, &tmp_dir, Some(&tag)).or_else(|_| {
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
pub(crate) fn try_http_download(root: &Path, folder: &str, version: &str) -> Result<(), Error> {
    let url = format!(
        "https://github.com/imazen/codec-corpus/releases/download/v{version}/{folder}.tar.gz"
    );

    let tmp_tar = temp_tar_path(root);
    let downloaded = download_file(&url, &tmp_tar);

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
        return Err(Error::Io(std::io::Error::other("tar extraction failed")));
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
// Generic file download (curl → wget → powershell fallback)
// ---------------------------------------------------------------------------

/// Download a file from `url` to `dest` using whatever HTTP client is
/// available on the system (curl, wget, or PowerShell).
pub(crate) fn download_file(url: &str, dest: &Path) -> Result<(), Error> {
    try_curl(url, dest)
        .or_else(|_| try_wget(url, dest))
        .or_else(|_| try_powershell(url, dest))
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
// Third-party download strategies
// ---------------------------------------------------------------------------

/// Git sparse checkout of a subfolder from an arbitrary GitHub repo.
///
/// Clones `https://github.com/{owner_repo}` with depth=1, sparse-checks out
/// `repo_path`, then moves the result into `dest_dir`.
pub(crate) fn git_sparse_checkout_github(
    dest_dir: &Path,
    owner_repo: &str,
    repo_path: &str,
    branch: Option<&str>,
) -> Result<(), Error> {
    if !has_git() {
        return Err(Error::NetworkUnavailable {
            dataset: owner_repo.to_string(),
        });
    }

    let repo_url = format!("https://github.com/{owner_repo}");
    let parent = dest_dir.parent().unwrap_or(dest_dir);
    let tmp_dir = temp_dir_path(parent);

    // Clone with the specified branch, or default
    let cloned = if let Some(b) = branch {
        try_git_clone(&repo_url, &tmp_dir, Some(b))
    } else {
        try_git_clone(&repo_url, &tmp_dir, None)
    };

    if cloned.is_err() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(Error::NetworkUnavailable {
            dataset: format!("{owner_repo}/{repo_path}"),
        });
    }

    // Sparse-checkout the specific path
    let sparse_result = Command::new("git")
        .args(["sparse-checkout", "set", &format!("{repo_path}/")])
        .current_dir(&tmp_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status();

    if !matches!(sparse_result, Ok(s) if s.success()) {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(Error::NetworkUnavailable {
            dataset: format!("{owner_repo}/{repo_path}"),
        });
    }

    let src = tmp_dir.join(repo_path);
    if src.is_dir() {
        // Ensure parent of dest exists
        if let Some(p) = dest_dir.parent() {
            std::fs::create_dir_all(p).map_err(Error::Io)?;
        }
        // Remove existing dest to avoid rename conflicts
        let _ = std::fs::remove_dir_all(dest_dir);
        std::fs::rename(&src, dest_dir).map_err(Error::Io)?;
    } else {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(Error::PathNotFound {
            path: format!("{owner_repo}/{repo_path}"),
        });
    }

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

/// Download a ZIP file from `url` and extract it into `dest_dir`.
///
/// Validates the downloaded file has ZIP magic bytes (`PK\x03\x04`) before
/// extraction. Uses shell `unzip` on Unix, `powershell Expand-Archive` on
/// Windows.
pub(crate) fn download_and_extract_zip(
    dest_dir: &Path,
    url: &str,
    _cache_key: &str,
) -> Result<(), Error> {
    let parent = dest_dir.parent().unwrap_or(dest_dir);
    std::fs::create_dir_all(parent).map_err(Error::Io)?;

    let tmp_zip = temp_zip_path(parent);

    let downloaded = try_curl(url, &tmp_zip)
        .or_else(|_| try_wget(url, &tmp_zip))
        .or_else(|_| try_powershell(url, &tmp_zip));

    if downloaded.is_err() {
        let _ = std::fs::remove_file(&tmp_zip);
        return Err(Error::DownloadFailed {
            url: url.to_string(),
            reason: "all download methods failed (curl, wget, powershell)".to_string(),
        });
    }

    // Validate ZIP magic bytes
    validate_zip_magic(&tmp_zip).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp_zip);
    })?;

    // Extract
    let tmp_extract = temp_dir_path(parent);
    std::fs::create_dir_all(&tmp_extract).map_err(Error::Io)?;

    let extract_ok = try_unzip(&tmp_zip, &tmp_extract)
        .or_else(|_| try_powershell_expand_archive(&tmp_zip, &tmp_extract));

    let _ = std::fs::remove_file(&tmp_zip);

    if extract_ok.is_err() {
        let _ = std::fs::remove_dir_all(&tmp_extract);
        return Err(Error::DownloadFailed {
            url: url.to_string(),
            reason: "ZIP extraction failed".to_string(),
        });
    }

    // Move extracted contents into dest_dir.
    // Some ZIPs extract into a single subdirectory, others directly into the
    // target. If there's exactly one subdirectory and no files, unwrap it.
    let _ = std::fs::remove_dir_all(dest_dir);
    let effective_src = unwrap_single_subdir(&tmp_extract);
    std::fs::rename(&effective_src, dest_dir)
        .or_else(|_| {
            // Cross-device rename — fall back to recursive copy
            copy_dir_recursive(&effective_src, dest_dir)
        })
        .map_err(|e| {
            let _ = std::fs::remove_dir_all(&tmp_extract);
            Error::Io(e)
        })?;

    let _ = std::fs::remove_dir_all(&tmp_extract);
    Ok(())
}

/// Download a tarball from `url` and extract it into `dest_dir`.
pub(crate) fn download_and_extract_tar(
    dest_dir: &Path,
    url: &str,
    _cache_key: &str,
) -> Result<(), Error> {
    let parent = dest_dir.parent().unwrap_or(dest_dir);
    std::fs::create_dir_all(parent).map_err(Error::Io)?;

    let tmp_tar = temp_tar_path(parent);

    let downloaded = try_curl(url, &tmp_tar)
        .or_else(|_| try_wget(url, &tmp_tar))
        .or_else(|_| try_powershell(url, &tmp_tar));

    if downloaded.is_err() {
        let _ = std::fs::remove_file(&tmp_tar);
        return Err(Error::DownloadFailed {
            url: url.to_string(),
            reason: "all download methods failed (curl, wget, powershell)".to_string(),
        });
    }

    let tmp_extract = temp_dir_path(parent);
    std::fs::create_dir_all(&tmp_extract).map_err(Error::Io)?;

    let tar_status = Command::new("tar")
        .args(["xf"])
        .arg(&tmp_tar)
        .arg("-C")
        .arg(&tmp_extract)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status();

    let _ = std::fs::remove_file(&tmp_tar);

    if !matches!(tar_status, Ok(s) if s.success()) {
        let _ = std::fs::remove_dir_all(&tmp_extract);
        return Err(Error::DownloadFailed {
            url: url.to_string(),
            reason: "tar extraction failed".to_string(),
        });
    }

    let _ = std::fs::remove_dir_all(dest_dir);
    let effective_src = unwrap_single_subdir(&tmp_extract);
    std::fs::rename(&effective_src, dest_dir)
        .or_else(|_| copy_dir_recursive(&effective_src, dest_dir))
        .map_err(|e| {
            let _ = std::fs::remove_dir_all(&tmp_extract);
            Error::Io(e)
        })?;

    let _ = std::fs::remove_dir_all(&tmp_extract);
    Ok(())
}

// ---------------------------------------------------------------------------
// ZIP helpers
// ---------------------------------------------------------------------------

/// Check that a file starts with ZIP magic bytes `PK\x03\x04`.
fn validate_zip_magic(path: &Path) -> Result<(), Error> {
    let mut f = std::fs::File::open(path).map_err(Error::Io)?;
    let mut magic = [0u8; 4];
    use std::io::Read;
    let n = f.read(&mut magic).map_err(Error::Io)?;
    if n >= 4 && magic == [0x50, 0x4B, 0x03, 0x04] {
        Ok(())
    } else {
        Err(Error::DownloadFailed {
            url: path.display().to_string(),
            reason: "downloaded file is not a valid ZIP (bad magic bytes)".to_string(),
        })
    }
}

/// Extract a ZIP using the `unzip` command.
fn try_unzip(zip_path: &Path, dest: &Path) -> Result<(), Error> {
    let status = Command::new("unzip")
        .args(["-q", "-o"])
        .arg(zip_path)
        .arg("-d")
        .arg(dest)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(Error::Io)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::Io(std::io::Error::other("unzip failed")))
    }
}

/// Extract a ZIP using PowerShell's `Expand-Archive` (Windows fallback).
fn try_powershell_expand_archive(zip_path: &Path, dest: &Path) -> Result<(), Error> {
    let script = "param($z,$d) Expand-Archive -Path $z -DestinationPath $d -Force";
    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .arg(zip_path)
        .arg(dest)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(Error::Io)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::Io(std::io::Error::other(
            "powershell Expand-Archive failed",
        )))
    }
}

/// If a directory contains exactly one subdirectory and no files, return
/// that subdirectory. Otherwise return the directory itself.
fn unwrap_single_subdir(dir: &Path) -> PathBuf {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return dir.to_path_buf();
    };
    let entries: Vec<_> = entries.flatten().collect();
    if entries.len() == 1 && entries[0].path().is_dir() {
        entries[0].path()
    } else {
        dir.to_path_buf()
    }
}

/// Recursively copy a directory tree. Used as fallback when `rename()` fails
/// (cross-device move).
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
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

fn temp_zip_path(root: &Path) -> std::path::PathBuf {
    let pid = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    root.join(format!(".tmp-{pid}-{ts}.zip"))
}
