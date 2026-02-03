# UPM Architecture

UPM is built around the idea of ​​a "unified interface + adapters".

## Main Components

### 1. UPM Core (`upm-core`)
Contains:
- API definitions for working with different package formats (backend traits)
- Universal data types (`Package`, `PackageQuery`, `Transaction`, errors)
- Logic for managing and maintaining the file database of installed packages
- Logic for installing and assigning permissions to source files, libraries, documentation, and manuals, i.e., the package contents, regardless of its format
- Logic for interacting with unpacked and converted "unpacked packages" in the usable format, for their subsequent installation into the system
- Logic for tracking, updating, and deleting repositories
- Logic for resolving dependencies of different formats for different packages
- Logic for downloading and retrieving packages from remote repositories

### 2. Backends
A backend is a module/plugin that implements support for a specific package format.

Examples of backends:
- `.deb` apt backend
- `.rpm` dnf backend
- `flatpak` flatpak backend
- `.pkg.tar.zst` pacman backend

The backend is responsible for:
- Unpacking the archive and converting the data within it into a format understood by UPM
- Obtaining installation paths, dependencies, and descriptions
- Building packages in an isolated environment
