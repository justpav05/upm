use crate::app::{AppResult};

use upac_core_lib::{Backend, Installer, Database, OStreeRepo, UpacConfig};

pub(crate) fn add(
    url: String,
) -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &Database,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |installer, ostree, config, database, backends| todo!()
}

pub(crate) fn remove(
    url: String,
) -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &Database,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |installer, ostree, config, database, backends| todo!()
}

pub(crate) fn update() -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &Database,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |installer, ostree, config, database, backends| todo!()
}
