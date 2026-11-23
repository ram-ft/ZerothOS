# Zeroth OS — Build Script (PowerShell)
# Builds the bare-metal Rust kernel and preps Limine boot image

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   Building Zeroth OS (x86_64)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$rootDir = Split-Path -Parent $PSScriptRoot
Set-Location $rootDir

# Check Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] Rust / Cargo is not installed on this system." -ForegroundColor Red
    Write-Host "Please install Rust from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host "[1/3] Compiling Zeroth Kernel (no_std)..." -ForegroundColor Green
cargo build --package zeroth-kernel --target x86_64-unknown-none --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "[FAIL] Kernel compilation failed." -ForegroundColor Red
    exit 1
}

$kernelBin = "$rootDir\target\x86_64-unknown-none\release\zeroth-kernel"
$isoDir = "$rootDir\target\iso_root"

Write-Host "[2/3] Assembling Bootloader & Kernel Files..." -ForegroundColor Green
New-Item -ItemType Directory -Force -Path "$isoDir\boot" | Out-Null
Copy-Item $kernelBin "$isoDir\boot\kernel.elf" -Force
Copy-Item "$rootDir\boot\limine.conf" "$isoDir\boot\limine.conf" -Force

Write-Host "[3/3] Zeroth OS Kernel ready for QEMU / Bare-Metal testing!" -ForegroundColor Green
Write-Host "Run .\scripts\run.ps1 to start in QEMU virtual machine." -ForegroundColor Yellow
