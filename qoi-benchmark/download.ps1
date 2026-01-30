# Download subsets from the QOI Benchmark Suite
# Source: https://qoiformat.org/benchmark/
#
# Usage:
#   .\download.ps1                            # Download all subsets (default)
#   .\download.ps1 -Subsets icon_512,icon_64  # Download specific subsets
#   .\download.ps1 -List                      # List available subsets
#
# Requires: PowerShell 5.1+ and tar (included in Windows 10+)

param(
    [string[]]$Subsets,
    [switch]$List
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TarUrl = "https://qoiformat.org/benchmark/qoi_benchmark_suite.tar"
$TmpDir = Join-Path $ScriptDir ".tmp_download"

$AllSubsets = @(
    "screenshot_web", "icon_512", "icon_64", "screenshot_game",
    "textures_pk", "textures_pk01", "textures_pk02", "textures_plants", "textures_photo",
    "photo_kodak", "photo_tecnick", "photo_wikipedia", "pngimg"
)

if ($List) {
    Write-Host "Available subsets:"
    foreach ($s in $AllSubsets) { Write-Host "  $s" }
    exit 0
}

if (-not $Subsets -or $Subsets.Count -eq 0) {
    $Subsets = $AllSubsets
}

Write-Host "Downloading QOI Benchmark Suite..."
Write-Host "This downloads the full ~1.1 GB tarball, then extracts selected subsets."
Write-Host "Subsets: $($Subsets -join ', ')"
Write-Host ""

if (-not (Test-Path $TmpDir)) {
    New-Item -ItemType Directory -Path $TmpDir | Out-Null
}

$TarFile = Join-Path $TmpDir "qoi_benchmark_suite.tar"

if (-not (Test-Path $TarFile)) {
    Write-Host "Downloading tarball..."
    Invoke-WebRequest -Uri $TarUrl -OutFile $TarFile
} else {
    Write-Host "Tarball already downloaded, reusing."
}

Write-Host ""
Write-Host "Extracting subsets..."

foreach ($subset in $Subsets) {
    $dest = Join-Path $ScriptDir $subset
    if (Test-Path $dest) {
        Write-Host "  ${subset}/ already exists, skipping."
        continue
    }
    New-Item -ItemType Directory -Path $dest | Out-Null

    $tempExtract = Join-Path $TmpDir "extract"
    if (Test-Path $tempExtract) { Remove-Item -Recurse -Force $tempExtract }
    New-Item -ItemType Directory -Path $tempExtract | Out-Null

    tar xf $TarFile -C $tempExtract "images/$subset/"

    $srcPath = Join-Path $tempExtract "images" $subset
    Get-ChildItem -Path $srcPath -File | Copy-Item -Destination $dest
    Remove-Item -Recurse -Force $tempExtract

    $count = (Get-ChildItem -Path $dest -File).Count
    Write-Host "  ${subset}/  ($count files)"
}

Write-Host ""
Write-Host "Cleaning up tarball..."
Remove-Item -Recurse -Force $TmpDir

Write-Host ""
Write-Host "Done. Extracted subsets:"
foreach ($subset in $Subsets) {
    $dest = Join-Path $ScriptDir $subset
    if (Test-Path $dest) {
        $size = (Get-ChildItem -Path $dest -Recurse -File | Measure-Object -Property Length -Sum).Sum
        $sizeMB = [math]::Round($size / 1MB, 1)
        Write-Host "  ${subset}/  ${sizeMB} MB"
    }
}
