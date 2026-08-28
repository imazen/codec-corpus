# r2-corpus

Command-line front end for [`codec_corpus::R2Corpus`](../README.md#public-r2-prefixes-r2corpus):
anonymous pull, authenticated push, `list`, `diff`, `login`, and a per-project
`corpus-sync.toml`. Tracks [imazen/codec-corpus#2](https://github.com/imazen/codec-corpus/issues/2).

```bash
cargo install --path crate/r2-corpus     # or: cargo run -p r2-corpus -- …

r2-corpus pull  fuzz/zentiff/                          # anonymous, whole subtree → cache path on stdout
r2-corpus pull  fuzz/zentiff/fuzz_decode/ --into fuzz/corpus/fuzz_decode
r2-corpus list  fuzz/zentiff/fuzz_decode/ [--json]     # show the .list
r2-corpus diff  fuzz/zentiff/fuzz_decode/ --local fuzz/corpus/fuzz_decode
r2-corpus login --endpoint https://<acct>.r2.cloudflarestorage.com --bucket codec-corpus
r2-corpus push  fuzz/zentiff/fuzz_decode/ --local fuzz/corpus/fuzz_decode [--rebundle]
r2-corpus sync  [--config corpus-sync.toml] [--push] [--dry-run]
```

Pull needs nothing but `curl`/`wget` (the library's own download chain). Push
needs the `aws` CLI on `PATH`, `tar` (+ `zstd` for `.tar.zst` bundles), and a
target from `r2-corpus login` (saved to `{config_dir}/codec-corpus/r2-push.json`,
mode 0600; override the path with `CODEC_CORPUS_R2_CONFIG`) or from
`CODEC_CORPUS_R2_ENDPOINT` / `CODEC_CORPUS_R2_BUCKET` + `AWS_ACCESS_KEY_ID` /
`AWS_SECRET_ACCESS_KEY` (or `R2_*`). `--dry-run` (and `diff`) need no
credentials and never upload.

## `corpus-sync.toml`

```toml
[corpus]
base_url = "https://codec-corpus.r2.imazen.org"   # optional (default)
local_dir = "fuzz/corpus"                          # base for relative `local`

[[sync]]
prefix = "fuzz/zentiff/fuzz_decode/"
local = "fuzz_decode"                              # → fuzz/corpus/fuzz_decode
```

`r2-corpus sync` pulls every entry into its `local` directory (listed files are
written when they differ; unlisted local files are left alone and counted so
new seeds are never deleted). `r2-corpus sync --push` pushes every `local`
directory to its prefix instead. Relative paths resolve against the config
file's directory.

Argument parsing is hand-rolled on `std`; `toml` is the only dependency beyond
the library, so the binary stays small.

## License

Apache-2.0.
