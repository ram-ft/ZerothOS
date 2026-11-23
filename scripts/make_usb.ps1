# Zeroth OS — Physical USB Boot Flash Helper
# Guides creating a bootable USB drive to run Zeroth OS on physical laptops and PCs

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  Zeroth OS — Bare-Metal Laptop / PC Boot Setup" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

Write-Host "To boot Zeroth OS on any physical laptop or PC (Dell, Lenovo ThinkPad, HP, Asus, etc.):`n"

Write-Host "Method 1: Rufus (Recommended for Windows):" -ForegroundColor Yellow
Write-Host "1. Download Rufus from https://rufus.ie/"
Write-Host "2. Plug in any USB flash drive (even a tiny 512MB / 1GB drive is enough!)."
Write-Host "3. In Rufus, select your USB drive and choose the Zeroth OS ISO/image."
Write-Host "4. Partition scheme: 'MBR' (Target system: BIOS or UEFI)."
Write-Host "5. Click Start!`n"

Write-Host "Method 2: Ventoy (Multi-boot USB):" -ForegroundColor Yellow
Write-Host "1. Install Ventoy on your USB from https://www.ventoy.net/"
Write-Host "2. Simply copy 'zeroth-os.iso' directly onto the USB."
Write-Host "3. Reboot your laptop, press F12 (or Esc/F9/F10 for Boot Menu), and select Zeroth OS!`n"
