use regex::Regex;
use std::num::NonZeroU64;

pub mod types;

trait UpacPkg {
    fn install(&self, package: &str);
    fn remove(&self, package: &str);
    fn update(&self, package: &str);
    fn search(&self, package: &str);
    fn upgrade(&self, package: &str);
}

trait UpacDisplay {
    fn metadata(&self, package: &str);
    fn files(&self, package: &str);
    fn dependencies(&self, package: &str);
    fn updependencies(&self, package: &str);
}

trait UpacRepo {
    fn add(&self, package: &str);
    fn remove(&self, package: &str);
    fn update(&self, package: &str);
    fn edit(&self, package: &str);
    fn rank(&self, package: &str);
}
#[derive(Default)]
struct InstallOptions {
    yes: bool,
    force: bool,
    download: bool,
}

#[derive(Default)]
struct RemoveOptions {
    yes: bool,
    recursive: bool,
    purge: bool,
    force: bool,
    dry_run: bool,
}

#[derive(Default)]
struct UpdateOptions {
    yes: bool,
    force: bool,
    no_deps: bool,
}

#[derive(Default)]
struct UpgdateOptions {
    yes: bool,
    force: bool,
    check_only: bool,
}

struct SearchOptions {
    exact: bool,
    description: bool,
    installed_only: bool,
    limit: Option<NonZeroU64>,
    regex: Option<Regex>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            exact: Default::default(),
            description: Default::default(),
            installed_only: Default::default(),
            limit: Default::default(),
            regex: None,
        }
    }
}
