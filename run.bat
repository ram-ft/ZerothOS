@echo off
setlocal
title Zeroth OS Launcher

echo =======================================================
echo   Launching Zeroth OS (Ultra-Lightweight x86_64)
echo =======================================================

rem Ensure cargo and qemu are in PATH
set "PATH=%PATH%;C:\Users\ram\.cargo\bin;C:\Program Files\qemu"

cd /d "%~dp0"

echo [1/3] Building Zeroth Kernel...
C:\Users\ram\.cargo\bin\cargo.exe build --package zeroth-kernel --target x86_64-unknown-none --release
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Kernel build failed!
    pause
    exit /b %ERRORLEVEL%
)

echo [2/3] Syncing UEFI Boot Files...
if not exist "%~dp0target\esp\EFI\BOOT" mkdir "%~dp0target\esp\EFI\BOOT"
if not exist "%~dp0target\esp\boot" mkdir "%~dp0target\esp\boot"
copy /Y "%~dp0boot\limine\limine-binary\BOOTX64.EFI" "%~dp0target\esp\EFI\BOOT\BOOTX64.EFI" >nul
copy /Y "%~dp0boot\limine.conf" "%~dp0target\esp\limine.conf" >nul
copy /Y "%~dp0target\x86_64-unknown-none\release\zeroth-kernel" "%~dp0target\esp\boot\kernel.elf" >nul

echo [3/3] Booting Zeroth OS in QEMU via UEFI...
echo.
echo =======================================================
echo  Zeroth OS Display Window Opening!
echo  (Press Ctrl+Alt+G in QEMU to release mouse cursor)
echo =======================================================
echo.

"C:\Program Files\qemu\qemu-system-x86_64.exe" ^
    -drive if=pflash,format=raw,readonly=on,file="C:\Program Files\qemu\share\edk2-x86_64-code.fd" ^
    -drive "file=fat:rw:%~dp0target\esp,format=raw" ^
    -m 256M ^
    -serial stdio ^
    -vga std

echo.
echo Zeroth OS session ended.
pause
