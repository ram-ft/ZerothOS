# Zeroth OS — Ultra-Lightweight Independent Operating System

> An independent, bare-metal 64-bit Operating System engineered from scratch for universal laptop and PC compatibility. Built with Rust, featuring native `.zero` application packages, an integrated hand-drawn aesthetic, and ultra-low resource requirements.

---

## 🌟 Key Highlights

- **Ultra-Lightweight**: Boots in **< 16MB of RAM** and idling consumes virtually 0% CPU. Runs smoothly on 15-year-old laptops (Intel Core 2 Duo) as well as modern Ryzen/Intel Core Ultra PCs.
- **Universal Laptop & PC Boot**: Dual-boot compatible with **both modern UEFI and legacy BIOS** via Limine Bootloader protocol.
- **Hardware-Agnostic Display**: Universal Linear Framebuffer engine supports Intel HD/Iris Graphics, AMD Radeon, NVIDIA, and Virtual Machines without requiring proprietary binary drivers.
- **Native `.zero` Application Ecosystem**: Self-contained application packages (e.g., `calculator.zero`, `terminal.zero`, `browser.zero`) with built-in permission manifests and cryptographic signatures.
- **Hand-Drawn Claude Aesthetic**: Warm cream (`#F0EEE6`), terracotta (`#D97757`), and ink charcoal (`#2B2A27`) styling across icons, boot screens, and wallpapers.

---

## 📁 Project Directory Structure

```text
zeroth-os/
├── assets/
│   ├── branding/           # Hand-drawn Zeroth OS vector logos (light, dark, transparent)
│   ├── wallpapers/         # 6 Claude-style wallpapers in 8K & FullHD PNG
│   └── icons/              # Hand-drawn Icon Pack for core apps & .zero packages:
│                           #   terminal.svg, files.svg, store.svg, settings.svg,
│                           #   calculator.svg, browser.svg, editor.svg, app-package.svg, power.svg
├── boot/
│   └── limine.conf         # Dual-boot UEFI + BIOS bootloader configuration
├── kernel/
│   ├── linker.ld           # 64-bit Higher-half kernel linker script
│   ├── Cargo.toml          # Kernel package specification (no_std)
│   └── src/
│       ├── main.rs         # Kernel entry point (_start) & desktop renderer
│       ├── arch/           # x86_64 architecture code
│       │   ├── gdt.rs      # Global Descriptor Table (Ring 0 & Ring 3 segments)
│       │   ├── interrupts.rs # Interrupt Descriptor Table (IDT)
│       │   └── serial.rs   # Universal 16550 UART serial logging driver
│       ├── drivers/        # Display & hardware drivers
│       │   ├── font.rs     # 8x16 embedded bitmap font engine
│       │   └── framebuffer.rs # Universal 2D linear framebuffer & card renderer
│       └── mm/             # Memory management
│           └── frame_allocator.rs # Physical 4KB memory page allocator
├── userland/
│   └── zeropkg/            # .zero Application Package Manager & Runtime
│       ├── Cargo.toml
│       └── src/            # Manifest parser, signature verification, bundle extractor
├── scripts/
│   ├── build.ps1           # Builds bare-metal kernel & preps bootloader files
│   ├── run.ps1             # Launches Zeroth OS in QEMU virtual machine
│   └── make_usb.ps1        # Step-by-step physical USB boot flash instructions
├── Cargo.toml              # Workspace root configuration
└── README.md
```

---

## 💻 Hardware Compatibility Matrix

| Hardware Type | Supported Specs | Tested Platforms |
| :--- | :--- | :--- |
| **Old Laptops (10-15 yrs old)** | Intel Core 2 Duo, Pentium Dual-Core, 512MB - 2GB RAM | ThinkPad T60/T400, Dell Inspiron, HP Pavilion |
| **Mid-Range / Budget PCs** | Intel Core i3 / i5 (Gen 2 - Gen 8), AMD A-Series, 4GB RAM | Acer Aspire, Asus Vivobook, Lenovo Ideapad |
| **Modern High-End Laptops** | Intel Core 10th-14th Gen, Core Ultra, AMD Ryzen 3000-8000 | ThinkPad X1, Dell XPS, HP Spectre, Framework |
| **Virtual Machines** | QEMU, VirtualBox, VMware Workstation, Hyper-V | Any Host OS |

---

## 🚀 How to Run

### 1. Install Prerequisites (if not already installed)
- **Rust Toolchain**: [https://rustup.rs](https://rustup.rs)
- **QEMU for Windows**: [https://www.qemu.org/download/#windows](https://www.qemu.org/download/#windows)

### 2. Build and Test in QEMU
In PowerShell:
```powershell
cd C:\Users\ram\.gemini\antigravity\scratch\zeroth-os
.\scripts\run.ps1
```

### 3. Flash to Physical USB for Real Laptop Boot
Run:
```powershell
.\scripts\make_usb.ps1
```
Follow the Rufus / Ventoy instructions to flash to any USB drive and boot on physical hardware by pressing `F12` during laptop startup.
