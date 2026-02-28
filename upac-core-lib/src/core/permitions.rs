// Imports
use nix::unistd::{Uid, Gid, chown};

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::fs;

// Function for setting file permissions and ownership
pub fn set_permissions(path: &Path, mode: u32, uid: u32, gid: u32) -> std::io::Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    chown(path, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid)))?;
    Ok(())
}
