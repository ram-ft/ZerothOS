# Zeroth OS Upgrade Plan – v0.2.0 → v0.3.0

## Goal
Create a **fully functional** set of desktop‑style applications that feel like a real OS while staying within the constraints of a bare‑metal, no‑std Rust kernel:
- **ZeroWeb** – a minimalist web browser capable of fetching HTTP(S) resources, parsing a tiny subset of HTML/CSS, and rendering text‑only pages.
- **ZeroTerminal** – the terminal we already built, now polished with scroll‑back, history, and a richer command set (done).
- **ZeroPad** – a hand‑drawn‑style text editor with cursor movement, insert/delete, line numbers, basic syntax highlighting for Rust/C/Python, and file‑like operations on an **in‑memory virtual file system**.

All UI components must keep the Claude hand‑drawn aesthetic (warm cream, terracotta, charcoal, ochre) and integrate with the existing window manager.

---

## Why a “real” browser is impossible now
A full‑featured browser (Chromium/Firefox) requires:
- A TCP/IP stack with routing, DNS, TLS, and sockets.
- A HTML/CSS/JS engine.
- Large memory and OS services (processes, sandboxing, etc.).
These cannot run on a tiny bare‑metal kernel without a substantial rewrite of the kernel itself.

**Solution:** Build a *mini‑browser* that can:
1. Use a **virtio‑net** driver to talk to the host network (QEMU provides a virtual NIC).
2. Implement a **tiny TCP/IP** stack (RIP‑based, static IP, no routing). Use existing Rust no‑std networking libraries as reference (e.g., smoltcp).
3. Add a **TLS 1.3** client (rustls in no‑std mode) for HTTPS.
4. Implement a **simple HTTP client** (GET only).
5. Parse **plain‑text HTML** – headings, paragraphs, links – and render them using the existing framebuffer API.
6. Provide a back/forward stack, address bar, and a minimal status bar.

This gives a functional web‑view experience suitable for documentation pages, README files, and simple sites.

---

## Text Editor – ZeroPad
Features to implement:
- **Buffer model**: a vector of `Line` structs (max 200 lines, each up to 256 bytes).
- **Cursor** with X/Y position, block cursor rendering.
- **Insert / Delete** handling, line wrap, backspace, Enter.
- **Navigation keys** – Arrow, Home/End, PageUp/PageDown, Ctrl‑S (save), Ctrl‑O (open).
- **File I/O** via a **virtual file system** stored in RAM (a simple `HashMap<&'static str, Vec<u8>>`). This sidesteps the lack of a real disk driver.
- **Syntax highlighting**: a small lexer that colors keywords (`fn`, `let`, `mut`, `struct`, `impl`, `pub`) for Rust, C, Python.
- **Claude hand‑drawn UI** – title bar, line numbers on the left, a bottom status bar showing `Ln X, Col Y` and file name.
- **Integration** with the window manager (minimize, drag, focus).

---

## Implementation Roadmap
| Step | Description | Approx. Effort |
|------|-------------|----------------|
| **1️⃣** | Add **virtio‑net PCI driver** (detect PCI device, map BAR, enable interrupts). | 2 days |
| **2️⃣** | Integrate **smoltcp** (no‑std) as the TCP/IP stack, configure static IP (e.g., 10.0.2.15). | 1 day |
| **3️⃣** | Add **rustls‑no‑std** for TLS 1.3 client. | 1 day |
| **4️⃣** | Implement **HTTP GET** helper (`http_get(url) -> Result<Vec<u8>>`). | 1 day |
| **5️⃣** | Create a **mini HTML parser** (tags: `<h1>`, `<h2>`, `<p>`, `<a href>`). Convert to a list of renderable items. | 2 days |
| **6️⃣** | Extend **ZeroWeb** UI to include address bar, back/forward buttons, simple scrollbars, and status line showing download progress. | 1 day |
| **7️⃣** | Build **virtual FS** (in‑memory map) with `read`, `write`, `list` APIs. | 0.5 day |
| **8️⃣** | Implement **ZeroPad** editor core (buffer, cursor, edit ops). | 2 days |
| **9️⃣** | Add **syntax highlight** for Rust/C/Python in ZeroPad. | 1 day |
| **🔟** | Polish UI – Claude hand‑drawn fonts, colors, window decorations, focus handling. | 1 day |
| **💡** | Testing & bug‑fixes (run QEMU, verify network, edit/save files). | 1 day |

Total estimated time: **~13 days** of development on a single workstation.

---

## Immediate Next Action (what we’ll do now)
1. **Create the virtio‑net driver** skeleton (PCI scan, BAR mapping). This will not yet be functional but sets up the hardware layer.
2. Add a **new module** `kernel/src/drivers/virtio_net.rs` and expose a `init()` function that will be called from `_start` after `init_ps2`.
3. Commit the driver file so you can review it.

Would you like me to proceed with step 1 now, or adjust any part of the plan? Let me know.

---

**Next steps you can trigger:**
- Run `/plan` to let us break down each step interactively.
- Run `/goal` if you want the system to work overnight until the full upgrade is finished.
- Or simply say “go ahead with the virtio‑net driver”.
