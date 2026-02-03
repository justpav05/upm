# UPM (Universal Package Manager)
UPM is a universal package manager that provides a single interface for installing, uninstalling, searching, and updating packages from different ecosystems. UPM is not tied to a single package manager: it uses a system of **backend adapters**, where each backend is responsible for working with a **specific package format** (or a specific ecosystem, like dep from debian like distors, or rpm from RHEL).

## Features

- Unified interface for package management
- Backend architecture: adding support for new formats without core modifications
- Abstract installation/uninstallation/search API
- Extensibility through Rust traits

## Building

Requirements:
- Rust (stable)
- Cargo

Build the workspace:

```bash
cargo build --release
```
