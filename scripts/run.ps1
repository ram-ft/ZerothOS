# Zeroth OS — QEMU Runner Script (PowerShell)
# Launches Zeroth OS in QEMU virtual machine via UEFI + Limine

$rootDir = Split-Path -Parent $PSScriptRoot
Set-Location $rootDir

$qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$uefiFirmware = "C:\Program Files\qemu\share\edk2-x86_64-code.fd"
$kernelBin = "$rootDir\target\x86_64-unknown-none\release\zeroth-kernel"
$espDir = "$rootDir\target\esp"

Write-Host "[1/3] Building Kernel..." -ForegroundColor Green
& "C:\Users\ram\.cargo\bin\cargo.exe" build --package zeroth-kernel --target x86_64-unknown-none --release
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "[2/3] Syncing UEFI boot files..." -ForegroundColor Green
New-Item -ItemType Directory -Force -Path "$espDir\EFI\BOOT" | Out-Null
New-Item -ItemType Directory -Force -Path "$espDir\boot" | Out-Null
Copy-Item "$rootDir\boot\limine\limine-binary\BOOTX64.EFI" "$espDir\EFI\BOOT\BOOTX64.EFI" -Force
Copy-Item "$rootDir\boot\limine.conf" "$espDir\limine.conf" -Force
Copy-Item $kernelBin "$espDir\boot\kernel.elf" -Force

Write-Host "[3/3] Launching Zeroth OS via UEFI..." -ForegroundColor Green
& $qemu `
    -drive if=pflash,format=raw,readonly=on,file=$uefiFirmware `
    -drive "file=fat:rw:$espDir,format=raw" `
    -m 256M `
    -serial stdio `
    -vga std
