use clap::{Parser, Subcommand, Args};

use upac_backend_alpm::AlpmBackend;

use commands::package;
use commands::repo;

use app::App;

use std::path::PathBuf;

mod app;
mod commands;

#[derive(Parser)]
#[command(name = "upac", about = "Universal package manager for Linux")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Install(InstallOptions),
    Remove(RemoveOptions),
    Update(UpdateOptions),
    Upgrade(UpgradeOptions),
    Search(SearchOptions),
    Show     { package: String },
    Files    { package: String },
    Deps     { package: String },
    #[command(subcommand)]
    Repo(RepoCommand),
}

#[derive(Subcommand)]
enum RepoCommand {
    Add    { url: String },
    Remove { url: String },
    Update,
}

#[derive(Args, Default)]
pub struct InstallOptions {
    pub package: PathBuf,
    #[arg(short, long)] pub yes:      bool,
    #[arg(short, long)] pub force:    bool,
    #[arg(short, long)] pub download: bool,
}

#[derive(Args, Default)]
pub struct RemoveOptions {
    pub package: String,
    #[arg(short, long)] pub yes:       bool,
    #[arg(short, long)] pub recursive: bool,
    #[arg(short, long)] pub purge:     bool,
    #[arg(short, long)] pub force:     bool,
    #[arg(short, long)] pub dry_run:   bool,
}

#[derive(Args, Default)]
pub struct UpdateOptions {
    pub package: PathBuf,
    #[arg(short, long)] pub yes:     bool,
    #[arg(short, long)] pub force:   bool,
    #[arg(short, long)] pub no_deps: bool,
}

#[derive(Args, Default)]
pub struct UpgradeOptions {
    #[arg(short, long)] pub yes:        bool,
    #[arg(short, long)] pub force:      bool,
    #[arg(short, long)] pub check_only: bool,
}

#[derive(Args, Default)]
pub struct SearchOptions {
    pub query: String,
    #[arg(short, long)] pub exact:          bool,
    #[arg(short, long)] pub description:    bool,
    #[arg(short, long)] pub installed_only: bool,
    #[arg(short, long)] pub limit:          Option<u64>,
}

fn main() {
	let cli = Cli::parse();

    let mut app = match App::init(vec![
        Box::new(AlpmBackend),
    ]) {
        Ok(app)  => app,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    let result = match cli.command {
        Command::Install(opts) => app.run(package::install(opts)),
        Command::Remove(opts)  => app.run(package::remove(opts)),
        Command::Update(opts)  => app.run(package::update(opts)),
        Command::Upgrade(opts) => app.run(package::upgrade(opts)),
        Command::Search(opts)  => app.run(package::search(opts)),
        Command::Show  { package } => app.run(package::show(&package)),
        Command::Files { package } => app.run(package::files(&package)),
        Command::Deps  { package } => app.run(package::deps(&package)),
        Command::Repo(cmd) => match cmd {
            RepoCommand::Add    { url } => app.run(repo::add(url)),
            RepoCommand::Remove { url } => app.run(repo::remove(url)),
            RepoCommand::Update        => app.run(repo::update()),
        },
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
