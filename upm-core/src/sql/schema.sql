-- Main packages table
CREATE TABLE IF NOT EXISTS packages (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT,
    repository TEXT,
    download_url TEXT,
    license TEXT,
    size_bytes INTEGER,
    installed BOOLEAN DEFAULT 0,
    installed_version TEXT,
    installed_time TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for packages table
CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);
CREATE INDEX IF NOT EXISTS idx_packages_backend ON packages(backend);
CREATE INDEX IF NOT EXISTS idx_packages_installed ON packages(installed);

-- Dependencies table
CREATE TABLE IF NOT EXISTS dependencies (
    id INTEGER PRIMARY KEY,
    package_id TEXT NOT NULL,
    dependency_id TEXT NOT NULL,
    version_constraint TEXT,
    is_optional BOOLEAN DEFAULT 0,
    FOREIGN KEY (package_id) REFERENCES packages(id),
    FOREIGN KEY (dependency_id) REFERENCES packages(id)
);

-- Operations table
CREATE TABLE IF NOT EXISTS operations (
    id TEXT PRIMARY KEY,
    operation_type TEXT NOT NULL,
    packages TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Snapshots table
CREATE TABLE IF NOT EXISTS snapshots (
    id TEXT PRIMARY KEY,
    commit_hash TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    size_bytes INTEGER,
    can_rollback BOOLEAN DEFAULT 1
);
