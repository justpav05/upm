pub struct PermissionsManager;

impl PermissionsManager {
    pub fn new() -> Self;

    // Ownership
    pub fn set_owner(&self, path: &Path, owner: &Owner) -> Result<()>;
    pub fn get_owner(&self, path: &Path) -> Result<Owner>;

    // Permissions
    pub fn set_permissions(&self, path: &Path, permissions: u32) -> Result<()>;
    pub fn get_permissions(&self, path: &Path) -> Result<u32>;

    // Group
    pub fn set_group(&self, path: &Path, group: &Group) -> Result<()>;
    pub fn get_group(&self, path: &Path) -> Result<Group>;

    // Verification
    pub fn verify_permissions(&self, path: &Path, expected: u32) -> Result<bool>;
    pub fn can_write(&self, path: &Path) -> bool;
    pub fn can_execute(&self, path: &Path) -> bool;
}

pub struct Owner {
    pub uid: u32,
    pub username: String,
}

pub struct Group {
    pub gid: u32,
    pub groupname: String,
}
