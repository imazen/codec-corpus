# QOI Benchmark Suite

Image subsets from the [QOI Benchmark Suite](https://qoiformat.org/benchmark/) by Dominic Szablewski.

## Subsets

| Subset | Files | Size | License |
|--------|-------|------|---------|
| `screenshot_web` | 15 | ~39 MB | CC0 1.0 |
| `icon_512` | 214 | ~12 MB | Public Domain (Tango Icons) |
| `icon_64` | 214 | ~1.3 MB | Public Domain (Tango Icons) |
| `screenshot_game` | 619 | ~256 MB | CC BY-SA 3.0 |
| `textures_pk` | 1004 | ~44 MB | — |
| `textures_pk01` | 115 | ~19 MB | — |
| `textures_pk02` | 237 | ~99 MB | — |
| `textures_plants` | 61 | ~50 MB | — |
| `textures_photo` | 21 | ~37 MB | — |
| `photo_kodak` | 25 | ~15 MB | — |
| `photo_tecnick` | 101 | ~228 MB | — |
| `photo_wikipedia` | 50 | ~85 MB | — |
| `pngimg` | 189 | ~220 MB | CC BY-NC 4.0 |

`screenshot_web` is committed directly to the repo. All other subsets can be fetched with the download scripts.

## Download

```bash
# Linux/macOS — download all subsets (~1.1 GB tarball, extracts ~1 GB)
./download.sh

# Download specific subsets only
./download.sh icon_512 icon_64

# List available subsets
./download.sh --list
```

```powershell
# Windows — download all subsets
.\download.ps1

# Download specific subsets
.\download.ps1 -Subsets icon_512,icon_64

# List available subsets
.\download.ps1 -List
```

The scripts download the full QOI benchmark tarball (~1.1 GB), extract the requested subsets, and delete the tarball.
