#[derive(Debug, Clone)]
pub struct Package {
    id: String,
    name: String,
    version: String,
    repository: String,
    state_of_instalation: bool,
    description: Option<String>,
    license: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum PackageField {
    Id,
    Name,
    Version,
    Description,
    Repository,
    License,
    StateOfInstalation,
}
