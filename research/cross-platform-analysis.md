# Cross-platform analysis

| Platform | Built by author | Run by author | CI | Status |
|---|---|---|---|---|
| Linux x86_64 (X11 via Xvfb, software GL) | Yes | Yes | configured | **Validated** (headless) |
| Linux Wayland | No | No | no | Not validated |
| Windows 10/11 | No | No | configured | **Not validated** |
| macOS | No | No | configured | **Not validated** |

Platform differences handled in code: per-OS directories (`directories`), portal-based dialogs
on Linux, `/proc` RSS only on Linux (other OS show "n/a"), `OPENRUST_HOME` for portable mode.

Known cross-platform risks (**Unverified**): Wayland decorations and clipboard, macOS
notarization requirements, Windows SmartScreen reputation for unsigned installers.
