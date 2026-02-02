pub struct PackageManager {
    config: PackageManagerConfig,

    database: Arc<Database>,

    thread_coordinator: Arc<ThreadCoordinator>,
    operation_manager: Arc<Mutex<OperationManager>>,

    dependency_resolver: DependencyResolver,
    cache_manager: CacheManager,
    snapshot_manager: SnapshotManager,

    backends: HashMap<String, Box<dyn BackendAdapter>>,

    event_bus: Arc<EventBus>,

    state: Arc<RwLock<PackageManagerState>>,
}

pub struct PackageManagerConfig {
    data_dir: PathBuf,
    temp_dir: PathBuf,
    cache_dir: PathBuf,
    config_file: PathBuf,

    database_path: PathBuf,

    max_parallel_downloads: usize,
    max_parallel_installs: usize,
    download_timeout: Duration,
    operation_timeout: Duration,

    cache_max_size: u64,
    cache_retention_days: u32,

    allow_untrusted: bool,
    verify_checksums: bool,
    create_snapshots: bool,

    log_level: LogLevel,
    log_file: Option<PathBuf>,
}
