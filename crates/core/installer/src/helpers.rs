use crate::Result;

use nix::unistd::{chown, Gid, Uid};

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(crate) fn set_permissions(path: &Path, mode: u32, uid: u32, gid: u32) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    chown(path, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid)))?;
    Ok(())
}
