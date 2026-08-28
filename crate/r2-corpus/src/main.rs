//! `r2-corpus`: command-line front end for [`codec_corpus::R2Corpus`]
//! (imazen/codec-corpus#2).
//!
//! ```text
//! r2-corpus pull  <prefix> [--into DIR]            anonymous, downloads the subtree
//! r2-corpus push  <prefix> --local DIR [--rebundle] authenticated (aws CLI)
//! r2-corpus list  <prefix> [--json]                show the .list
//! r2-corpus diff  <prefix> --local DIR             local vs remote, no upload
//! r2-corpus login --endpoint URL --bucket NAME     store R2 credentials
//! r2-corpus sync  [--config corpus-sync.toml] [--push]
//! ```
//!
//! Argument parsing is hand-rolled on `std` so the binary adds no CLI
//! framework; `toml` is the only dependency beyond the library.

#![forbid(unsafe_code)]

use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use codec_corpus::{
    BundleFormat, DEFAULT_R2_BASE_URL, ListIndex, PullMode, PullOptions, PushOptions, PushReport,
    PushTarget, R2Corpus, Rebundle,
};
use serde::Deserialize;

const USAGE: &str = "\
r2-corpus — sync corpus prefixes with a public-read R2 bucket

USAGE:
  r2-corpus pull  <prefix> [--into DIR] [--base-url URL] [--cache-root DIR]
                  [--mode auto|bundle|no-bundle|offline] [--parallelism N]
                  [--no-recursive] [--no-prune]
  r2-corpus push  <prefix> --local DIR [--rebundle | --no-bundle | --max-deltas N]
                  [--format zst|gz|tar] [--dry-run] [--no-register-parents]
                  [--parallelism N] [--base-url URL]
  r2-corpus list  <prefix> [--base-url URL] [--json]
  r2-corpus diff  <prefix> --local DIR [--base-url URL]
  r2-corpus login --endpoint URL --bucket NAME [--base-url URL]
                  [--access-key-id ID --secret-access-key KEY]   (else prompted)
  r2-corpus sync  [--config corpus-sync.toml] [--push] [--dry-run]

`pull` mirrors the prefix into the codec-corpus cache and prints its path;
with --into DIR it also copies every listed file into DIR (files in DIR that
are not listed are left alone and reported). `push` needs `aws` on PATH and a
target from `r2-corpus login` or CODEC_CORPUS_R2_ENDPOINT /
CODEC_CORPUS_R2_BUCKET + AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY.

corpus-sync.toml:
  [corpus]
  base_url = \"https://codec-corpus.r2.imazen.org\"   # optional
  local_dir = \"fuzz/corpus\"                          # base for relative `local`
  [[sync]]
  prefix = \"fuzz/zentiff/fuzz_decode/\"
  local = \"fuzz_decode\"
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("r2-corpus: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let Some(cmd) = args.first() else {
        eprint!("{USAGE}");
        return Err("no command given".into());
    };
    let rest = &args[1..];
    match cmd.as_str() {
        "pull" => cmd_pull(rest),
        "push" => cmd_push(rest),
        "list" => cmd_list(rest),
        "diff" => cmd_diff(rest),
        "login" => cmd_login(rest),
        "sync" => cmd_sync(rest),
        "-h" | "--help" | "help" => {
            print!("{USAGE}");
            Ok(())
        }
        other => {
            eprint!("{USAGE}");
            Err(format!("unknown command '{other}'").into())
        }
    }
}

// ---------------------------------------------------------------------------
// Minimal flag parser
// ---------------------------------------------------------------------------

/// `--key value` options, `--flag` switches, and bare positionals.
struct Parsed {
    positional: Vec<String>,
    values: Vec<(String, String)>,
    flags: Vec<String>,
}

fn parse(args: &[String], value_keys: &[&str], flag_keys: &[&str]) -> Result<Parsed, String> {
    let mut p = Parsed {
        positional: Vec::new(),
        values: Vec::new(),
        flags: Vec::new(),
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if let Some(key) = a.strip_prefix("--") {
            let (key, inline) = match key.split_once('=') {
                Some((k, v)) => (k, Some(v.to_string())),
                None => (key, None),
            };
            if value_keys.contains(&key) {
                let v = match inline {
                    Some(v) => v,
                    None => it
                        .next()
                        .cloned()
                        .ok_or_else(|| format!("--{key} needs a value"))?,
                };
                p.values.push((key.to_string(), v));
            } else if flag_keys.contains(&key) {
                p.flags.push(key.to_string());
            } else {
                return Err(format!("unknown option --{key}"));
            }
        } else {
            p.positional.push(a.clone());
        }
    }
    Ok(p)
}

impl Parsed {
    fn value(&self, key: &str) -> Option<&str> {
        self.values
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
    fn flag(&self, key: &str) -> bool {
        self.flags.iter().any(|f| f == key)
    }
    fn prefix(&self) -> Result<&str, String> {
        self.positional
            .first()
            .map(String::as_str)
            .ok_or_else(|| "missing <prefix>".to_string())
    }
    fn base_url(&self) -> String {
        self.value("base-url")
            .map(str::to_string)
            .or_else(|| std::env::var("CODEC_CORPUS_R2_BASE_URL").ok())
            .unwrap_or_else(|| DEFAULT_R2_BASE_URL.to_string())
    }
    fn usize_value(&self, key: &str) -> Result<Option<usize>, String> {
        self.value(key)
            .map(|v| {
                v.parse::<usize>()
                    .map_err(|_| format!("--{key} wants a number"))
            })
            .transpose()
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

fn pull_options(p: &Parsed) -> Result<PullOptions, String> {
    let mut o = PullOptions::default();
    if let Some(mode) = p.value("mode") {
        o = o.mode(match mode {
            "auto" => PullMode::Auto,
            "bundle" | "force-bundle" => PullMode::ForceBundle,
            "no-bundle" => PullMode::NoBundle,
            "offline" => PullMode::Offline,
            other => return Err(format!("unknown --mode '{other}'")),
        });
    }
    if let Some(root) = p.value("cache-root") {
        o = o.cache_root(root);
    }
    if let Some(n) = p.usize_value("parallelism")? {
        o = o.parallelism(n);
    }
    if p.flag("no-recursive") {
        o = o.recursive(false);
    }
    if p.flag("no-prune") {
        o = o.prune(false);
    }
    Ok(o)
}

fn cmd_pull(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let p = parse(
        args,
        &["into", "base-url", "cache-root", "mode", "parallelism"],
        &["no-recursive", "no-prune"],
    )?;
    let prefix = p.prefix()?;
    let corpus = R2Corpus::pull_with_options(&p.base_url(), prefix, pull_options(&p)?)?;
    eprintln!(
        "pulled {} ({} files, {} bytes)",
        corpus.prefix(),
        corpus.list().len(),
        corpus.list().total_size()
    );
    if let Some(into) = p.value("into") {
        let (copied, extras) = mirror_into(&corpus, Path::new(into))?;
        eprintln!("mirrored into {into}: {copied} files written");
        for extra in extras {
            eprintln!("  not listed remotely (left alone): {extra}");
        }
    }
    println!("{}", corpus.path().display());
    Ok(())
}

/// Copy every listed file of `corpus` into `dir` (only when the local copy
/// differs by size). Returns `(files written, unlisted files found in dir)`.
fn mirror_into(corpus: &R2Corpus, dir: &Path) -> Result<(usize, Vec<String>), std::io::Error> {
    std::fs::create_dir_all(dir)?;
    let mut copied = 0usize;
    for (rel, entry) in corpus.files() {
        let dest = rel.split('/').fold(dir.to_path_buf(), |d, c| d.join(c));
        let same = std::fs::metadata(&dest)
            .map(|m| m.is_file() && m.len() == entry.size)
            .unwrap_or(false);
        if same && std::fs::read(&dest)? == std::fs::read(corpus.file_path(rel))? {
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(corpus.file_path(rel), &dest)?;
        copied += 1;
    }
    let listed: std::collections::BTreeSet<&str> = corpus.files().map(|(k, _)| k).collect();
    let mut extras = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d)? {
            let path = e?.path();
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path
                    .strip_prefix(dir)
                    .map(|r| {
                        r.components()
                            .map(|c| c.as_os_str().to_string_lossy().into_owned())
                            .collect::<Vec<_>>()
                            .join("/")
                    })
                    .unwrap_or_default();
                if !listed.contains(rel.as_str()) {
                    extras.push(rel);
                }
            }
        }
    }
    extras.sort();
    Ok((copied, extras))
}

fn push_options(p: &Parsed) -> Result<PushOptions, String> {
    let mut o = PushOptions::default();
    if p.flag("rebundle") {
        o = o.rebundle(Rebundle::Always);
    } else if p.flag("no-bundle") {
        o = o.rebundle(Rebundle::Never);
    } else if let Some(n) = p.usize_value("max-deltas")? {
        o = o.rebundle(Rebundle::Auto { max_deltas: n });
    }
    if let Some(f) = p.value("format") {
        o = o.bundle_format(match f {
            "zst" | "tar.zst" => BundleFormat::TarZst,
            "gz" | "tar.gz" => BundleFormat::TarGz,
            "tar" => BundleFormat::Tar,
            other => return Err(format!("unknown --format '{other}'")),
        });
    }
    if p.flag("dry-run") {
        o = o.dry_run(true);
    }
    if p.flag("no-register-parents") {
        o = o.register_parents(false);
    }
    if let Some(n) = p.usize_value("parallelism")? {
        o = o.parallelism(n);
    }
    Ok(o)
}

fn print_report(r: &PushReport) {
    let verb = if r.dry_run {
        "would upload"
    } else {
        "uploaded"
    };
    println!(
        "{}: {verb} {} object(s), {} unchanged, {} removed from list{}",
        r.prefix,
        r.uploaded.len(),
        r.unchanged,
        r.removed.len(),
        if r.list_uploaded {
            ", .list updated"
        } else {
            ""
        }
    );
    for rel in &r.uploaded {
        println!("  + {rel}");
    }
    for rel in &r.removed {
        println!("  - {rel}");
    }
    if let Some(b) = &r.bundled {
        println!(
            "  bundle {} ({} files, {} bytes uncompressed)",
            b.key, b.file_count, b.uncompressed_size
        );
    }
    for parent in &r.parents_updated {
        println!("  registered in {}.list", parent.trim_end_matches('/'));
    }
}

fn cmd_push(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let p = parse(
        args,
        &["local", "max-deltas", "format", "parallelism", "base-url"],
        &["rebundle", "no-bundle", "dry-run", "no-register-parents"],
    )?;
    let prefix = p.prefix()?;
    let local = p.value("local").ok_or("push needs --local DIR")?;
    let options = push_options(&p)?;
    let report = if p.flag("dry-run") {
        R2Corpus::diff(Path::new(local), &p.base_url(), prefix, options)?
    } else {
        let mut target = PushTarget::load()?;
        if let Some(b) = p.value("base-url") {
            target.base_url = b.to_string();
        }
        R2Corpus::push(Path::new(local), prefix, &target, options)?
    };
    print_report(&report);
    Ok(())
}

fn cmd_list(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let p = parse(args, &["base-url"], &["json"])?;
    let prefix = p.prefix()?;
    let list = R2Corpus::fetch_list(&p.base_url(), prefix)?;
    if p.flag("json") {
        println!("{}", list.to_json());
        return Ok(());
    }
    print_list(&list);
    Ok(())
}

fn print_list(list: &ListIndex) {
    println!(
        "{} — {} files, {} bytes, generated {}",
        list.prefix,
        list.len(),
        list.total_size(),
        list.generated_at
    );
    if let Some(b) = &list.bundle {
        println!(
            "bundle: {} ({} bytes, {} files, sha256 {})",
            b.key, b.size, b.file_count, b.sha256
        );
    }
    for child in &list.children {
        println!("child: {child}/");
    }
    for (rel, e) in &list.files {
        println!(
            "{:>10}  {}  {}{}",
            e.size,
            &e.sha256[..12.min(e.sha256.len())],
            rel,
            if e.in_bundle { "" } else { "  (delta)" }
        );
    }
}

fn cmd_diff(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let p = parse(args, &["local", "base-url"], &[])?;
    let prefix = p.prefix()?;
    let local = p.value("local").ok_or("diff needs --local DIR")?;
    let report = R2Corpus::diff(
        Path::new(local),
        &p.base_url(),
        prefix,
        PushOptions::default().rebundle(Rebundle::Never),
    )?;
    print_report(&report);
    Ok(())
}

fn cmd_login(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let p = parse(
        args,
        &[
            "endpoint",
            "bucket",
            "base-url",
            "access-key-id",
            "secret-access-key",
        ],
        &[],
    )?;
    let endpoint_url = p.value("endpoint").ok_or("login needs --endpoint URL")?;
    let bucket = p.value("bucket").ok_or("login needs --bucket NAME")?;
    let access_key_id = match p.value("access-key-id") {
        Some(v) => v.to_string(),
        None => prompt("R2 access key id: ")?,
    };
    let secret_access_key = match p.value("secret-access-key") {
        Some(v) => v.to_string(),
        None => prompt("R2 secret access key: ")?,
    };
    let target = PushTarget {
        endpoint_url: endpoint_url.to_string(),
        bucket: bucket.to_string(),
        base_url: p.base_url(),
        access_key_id: Some(access_key_id).filter(|s| !s.is_empty()),
        secret_access_key: Some(secret_access_key).filter(|s| !s.is_empty()),
    };
    let path = target.save()?;
    println!("saved push target to {}", path.display());
    Ok(())
}

/// Read one line from stdin (interactive prompt when stdin is a terminal).
fn prompt(label: &str) -> Result<String, std::io::Error> {
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        eprint!("{label}");
        std::io::stderr().flush()?;
    }
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

// ---------------------------------------------------------------------------
// corpus-sync.toml
// ---------------------------------------------------------------------------

/// Per-project sync configuration.
#[derive(Debug, Deserialize)]
struct SyncConfig {
    #[serde(default)]
    corpus: CorpusSection,
    #[serde(default)]
    sync: Vec<SyncEntry>,
}

#[derive(Debug, Default, Deserialize)]
struct CorpusSection {
    base_url: Option<String>,
    local_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct SyncEntry {
    prefix: String,
    local: PathBuf,
}

fn load_sync_config(path: &Path) -> Result<SyncConfig, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let cfg: SyncConfig = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if cfg.sync.is_empty() {
        return Err(format!("{}: no [[sync]] entries", path.display()).into());
    }
    Ok(cfg)
}

fn cmd_sync(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let p = parse(
        args,
        &["config", "cache-root", "parallelism"],
        &["push", "dry-run"],
    )?;
    let config_path = PathBuf::from(p.value("config").unwrap_or("corpus-sync.toml"));
    let cfg = load_sync_config(&config_path)?;
    let config_dir = config_path
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let base_url = cfg.corpus.base_url.clone().unwrap_or_else(|| p.base_url());
    let base_dir = match &cfg.corpus.local_dir {
        Some(d) if d.is_absolute() => d.clone(),
        Some(d) => config_dir.join(d),
        None => config_dir.clone(),
    };
    let mut failures = 0usize;
    for entry in &cfg.sync {
        let local = if entry.local.is_absolute() {
            entry.local.clone()
        } else {
            base_dir.join(&entry.local)
        };
        let result: Result<(), Box<dyn std::error::Error>> = if p.flag("push") {
            let options = PushOptions::default().dry_run(p.flag("dry-run"));
            let report = if p.flag("dry-run") {
                R2Corpus::diff(&local, &base_url, &entry.prefix, options)
            } else {
                let mut target = PushTarget::load()?;
                target.base_url = base_url.clone();
                R2Corpus::push(&local, &entry.prefix, &target, options)
            };
            report.map(|r| print_report(&r)).map_err(Into::into)
        } else {
            let options = pull_options(&p)?;
            R2Corpus::pull_with_options(&base_url, &entry.prefix, options)
                .map_err(Into::into)
                .and_then(|corpus| {
                    let (copied, extras) = mirror_into(&corpus, &local)?;
                    println!(
                        "{} -> {}: {} files written, {} unlisted local file(s)",
                        corpus.prefix(),
                        local.display(),
                        copied,
                        extras.len()
                    );
                    Ok(())
                })
        };
        if let Err(e) = result {
            failures += 1;
            eprintln!("{}: {e}", entry.prefix);
        }
    }
    if failures > 0 {
        return Err(format!(
            "{failures} sync entr{} failed",
            if failures == 1 { "y" } else { "ies" }
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &[&str]) -> Vec<String> {
        s.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn parser_handles_values_flags_positionals_and_inline_eq() {
        let p = parse(
            &args(&["fuzz/x/", "--into", "d", "--mode=offline", "--no-prune"]),
            &["into", "mode"],
            &["no-prune"],
        )
        .unwrap();
        assert_eq!(p.prefix().unwrap(), "fuzz/x/");
        assert_eq!(p.value("into"), Some("d"));
        assert_eq!(p.value("mode"), Some("offline"));
        assert!(p.flag("no-prune"));
        assert!(!p.flag("no-recursive"));
        assert!(parse(&args(&["--bogus"]), &[], &[]).is_err());
        assert!(parse(&args(&["--into"]), &["into"], &[]).is_err());
        assert!(parse(&args(&[]), &[], &[]).unwrap().prefix().is_err());
    }

    #[test]
    fn push_option_flags_map_to_policies() {
        let p = parse(
            &args(&["p/", "--rebundle", "--format", "gz", "--dry-run"]),
            &["format"],
            &["rebundle", "dry-run"],
        )
        .unwrap();
        assert!(push_options(&p).is_ok());
        let bad = parse(&args(&["p/", "--format", "rar"]), &["format"], &[]).unwrap();
        assert!(push_options(&bad).is_err());
        let bad = parse(&args(&["p/", "--max-deltas", "x"]), &["max-deltas"], &[]).unwrap();
        assert!(push_options(&bad).is_err());
    }

    #[test]
    fn sync_config_parses_the_documented_shape() {
        let cfg: SyncConfig = toml::from_str(
            r#"
            [corpus]
            base_url = "https://corpus.example"
            local_dir = "fuzz/corpus"

            [[sync]]
            prefix = "fuzz/zentiff/fuzz_decode/"
            local = "fuzz_decode"

            [[sync]]
            prefix = "fuzz/zentiff/fuzz_encode/"
            local = "/abs/encode"
            "#,
        )
        .unwrap();
        assert_eq!(
            cfg.corpus.base_url.as_deref(),
            Some("https://corpus.example")
        );
        assert_eq!(
            cfg.corpus.local_dir.as_deref(),
            Some(Path::new("fuzz/corpus"))
        );
        assert_eq!(cfg.sync.len(), 2);
        assert_eq!(cfg.sync[0].prefix, "fuzz/zentiff/fuzz_decode/");
        assert_eq!(cfg.sync[1].local, Path::new("/abs/encode"));
        // Minimal form: no [corpus] table at all.
        let min: SyncConfig = toml::from_str("[[sync]]\nprefix = \"a/\"\nlocal = \"a\"\n").unwrap();
        assert!(min.corpus.base_url.is_none());
        assert_eq!(min.sync.len(), 1);
    }
}
