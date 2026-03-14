use upac_core_lib::{
    Backend, Config, Database, InstallEvent, Installer, OStreeError, OStreeRepo, PackageDatabase,
    UpacConfig,
};

use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    InitError(String),
    CommandError(String),
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::InitError(err.to_string())
    }
}

impl From<OStreeError> for AppError {
    fn from(err: OStreeError) -> Self {
        AppError::CommandError(err.to_string())
    }
}

impl From<InstallerError> for AppError {
    fn from(err: InstallerError) -> Self {
        AppError::CommandError(err.to_string())
    }
}


impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InitError(msg) => write!(f, "Initialization error: {msg}"),
            AppError::CommandError(msg) => write!(f, "Command error: {msg}"),
        }
    }
}

pub enum ErrorMessage {
    Error(String),
    Shutdown,
}

pub struct App {
    config: UpacConfig,
    installer: Installer,
    ostree: Option<OStreeRepo>,
    database: Database,
    error_tx: Sender<ErrorMessage>,
    backends: Vec<Box<dyn Backend>>,
}

impl App {
    pub fn init(backends: Vec<Box<dyn Backend>>) -> AppResult<Self> {
        let (install_tx, install_rx) = channel();
        let (error_tx, error_rx) = channel::<ErrorMessage>();

        let config = UpacConfig::load().map_err(|err| AppError::InitError(err.to_string()))?;

        let database = match Database::new(config.database_path.clone()) {
            Ok(database) => database,
            Err(err) => return Err(AppError::InitError(err.to_string())),
        };

        let _ = Self::spawn_error_thread(error_rx)?;
        let _ = Self::spawn_installer_message_thread(install_rx)?;

        let boxed_database: Box<dyn PackageDatabase> = Box::new(database.clone());

        let installer = Installer::new(
            boxed_database,
            config.ostree.enabled,
            config.root_dir.clone(),
            config.package_dir.clone(),
            config.temp_dir.clone(),
            install_tx,
        );

        let ostree = if config.ostree.enabled {
            Some(
                OStreeRepo::open(config.ostree.repo_path.clone())
                    .map_err(|err| AppError::InitError(err.to_string()))?,
            )
        } else {
            None
        };

        Ok(Self {
            installer,
            ostree,
            config,
            database,
            error_tx,
            backends,
        })
    }

    fn spawn_installer_message_thread(
        install_rx: Receiver<InstallEvent>,
    ) -> AppResult<JoinHandle<()>> {
        let installer_thread = thread::Builder::new()
            .name(String::from("installer"))
            .spawn(move || {
                for message in install_rx {
                    match message {
                        InstallEvent::InstallStarted {
                            package,
                            total_files,
                        } => println!("Installing {} ({} files)...", package, total_files),
                        InstallEvent::FileInstalled { current, total, .. } => {
                            print!("\rProgress: {}/{}", current, total)
                        }
                        InstallEvent::InstallFinished { package } => {
                            println!("\nDone: {} installed", package)
                        }
                        InstallEvent::RemoveStarted { package } => {
                            println!("Removing {}...", package)
                        }
                        InstallEvent::RemoveFinished { package } => {
                            println!("Done: {} removed", package)
                        }
                        InstallEvent::Failed { package, reason } => {
                            eprintln!("Failed {}: {}", package, reason)
                        }
                        _ => {}
                    }
                }
            })?;

        Ok(installer_thread)
    }

    fn spawn_error_thread(error_rx: Receiver<ErrorMessage>) -> AppResult<JoinHandle<()>> {
        let error_thread = thread::Builder::new()
            .name(String::from("error"))
            .spawn(move || {
                for message in error_rx {
                    match message {
                        ErrorMessage::Error(msg) => eprintln!("Error: {msg}"),
                        ErrorMessage::Shutdown => break,
                    }
                }
            })?;

        Ok(error_thread)
    }

    pub fn run<F>(&mut self, command: F) -> AppResult<()>
    where
        F: FnOnce(
            &mut Installer,
            Option<&OStreeRepo>,
            &UpacConfig,
            &Database,
            &[Box<dyn Backend>],
        ) -> AppResult<()>,
    {
        command(
            &mut self.installer,
            self.ostree.as_ref(),
            &self.config,
            &self.database,
            &self.backends,
        )
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.error_tx.send(ErrorMessage::Shutdown);
    }
}
