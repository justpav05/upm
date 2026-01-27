//! APT Backend для UPM
//!
//! Реализация PackageBackend для работы с apt (Debian/Ubuntu).
//! Компилируется как динамическая библиотека (cdylib).

use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use upm_core::backend::{
    BackendError, BackendOperationResult, BackendPackageInfo, BackendStatus, PackageBackend,
};

// ============================================================================
// APT Backend
// ============================================================================

/// Бэкенд для работы с APT (Debian/Ubuntu)
pub struct AptBackend {
    /// Кэш для проверки доступности
    available: bool,
}

impl AptBackend {
    /// Создаёт новый экземпляр APT бэкенда
    pub fn new() -> Self {
        Self { available: false }
    }

    /// Инициализирует бэкенд и проверяет доступность
    async fn init(&mut self) {
        self.available = self.check_apt_available().await;
    }

    /// Проверяет, установлен ли apt в системе
    async fn check_apt_available(&self) -> bool {
        Command::new("which")
            .arg("apt")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Выполняет команду apt
    async fn run_apt_command(&self, args: &[&str]) -> Result<BackendOperationResult, BackendError> {
        log::debug!("Running apt command: apt {}", args.join(" "));

        let mut cmd = Command::new("apt");
        cmd.args(args)
            .env("DEBIAN_FRONTEND", "noninteractive") // Неинтерактивный режим
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| BackendError::Unknown(format!("Failed to execute apt command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();

        log::debug!("APT exit code: {:?}", exit_code);
        log::debug!("APT stdout: {}", stdout);
        if !stderr.is_empty() {
            log::debug!("APT stderr: {}", stderr);
        }

        Ok(BackendOperationResult {
            success: output.status.success(),
            error: if output.status.success() {
                None
            } else {
                Some(stderr.clone())
            },
            stdout: Some(stdout),
            stderr: Some(stderr),
            exit_code,
        })
    }

    /// Парсит вывод `apt show` в структуру PackageInfo
    fn parse_apt_show(&self, output: &str) -> Result<BackendPackageInfo, BackendError> {
        let mut name = String::new();
        let mut version = String::new();
        let mut description = None;
        let mut size = None;
        let mut dependencies = Vec::new();
        let mut metadata = HashMap::new();

        for line in output.lines() {
            if line.starts_with("Package:") {
                name = line.trim_start_matches("Package:").trim().to_string();
            } else if line.starts_with("Version:") {
                version = line.trim_start_matches("Version:").trim().to_string();
            } else if line.starts_with("Description:") {
                description = Some(line.trim_start_matches("Description:").trim().to_string());
            } else if line.starts_with("Installed-Size:") {
                if let Ok(kb) = line
                    .trim_start_matches("Installed-Size:")
                    .trim()
                    .parse::<u64>()
                {
                    size = Some(kb * 1024); // Конвертируем KB в байты
                }
            } else if line.starts_with("Depends:") {
                let deps_str = line.trim_start_matches("Depends:").trim();
                dependencies = deps_str
                    .split(',')
                    .map(|d| {
                        // Убираем версионные ограничения вида (>= 1.0)
                        d.trim().split_whitespace().next().unwrap_or("").to_string()
                    })
                    .filter(|d| !d.is_empty())
                    .collect();
            } else if line.starts_with("Section:") {
                metadata.insert(
                    "section".to_string(),
                    line.trim_start_matches("Section:").trim().to_string(),
                );
            }
        }

        if name.is_empty() {
            return Err(BackendError::PackageNotFound(
                "Failed to parse package name".to_string(),
            ));
        }

        Ok(BackendPackageInfo {
            name,
            version,
            repository: "apt".to_string(),
            description,
            license: None,
            size,
            dependencies,
            metadata,
        })
    }

    /// Получает версию APT
    async fn get_apt_version(&self) -> Option<String> {
        let output = Command::new("apt").arg("--version").output().await.ok()?;

        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            // Парсим "apt 2.4.5 (amd64)" -> "2.4.5"
            let re = Regex::new(r"apt\s+(\d+\.\d+\.\d+)").ok()?;
            re.captures(&version_str)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
        } else {
            None
        }
    }
}

impl Default for AptBackend {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PackageBackend Implementation
// ============================================================================

#[async_trait]
impl PackageBackend for AptBackend {
    fn name(&self) -> &str {
        "apt"
    }

    async fn is_available(&self) -> bool {
        self.check_apt_available().await
    }

    async fn get_status(&self) -> Result<BackendStatus, BackendError> {
        let available = self.is_available().await;
        let version = if available {
            self.get_apt_version().await
        } else {
            None
        };

        Ok(BackendStatus {
            available,
            version,
            info: Some("APT package manager for Debian/Ubuntu".to_string()),
        })
    }

    async fn update_cache(&self) -> Result<BackendOperationResult, BackendError> {
        if !self.available {
            return Err(BackendError::NotAvailable(
                "APT is not available on this system".to_string(),
            ));
        }

        self.run_apt_command(&["update"]).await
    }

    async fn search_package(&self, name: &str) -> Result<Option<BackendPackageInfo>, BackendError> {
        if !self.available {
            return Err(BackendError::NotAvailable(
                "APT is not available on this system".to_string(),
            ));
        }

        // Используем apt-cache search для поиска
        let result = Command::new("apt-cache")
            .args(&["search", name])
            .output()
            .await
            .map_err(|e| BackendError::Unknown(format!("Failed to search: {}", e)))?;

        if !result.status.success() {
            return Ok(None);
        }

        let output = String::from_utf8_lossy(&result.stdout);
        if output.trim().is_empty() {
            return Ok(None);
        }

        // Если нашли, получаем полную информацию
        self.get_package_info(name).await.map(Some)
    }

    async fn get_package_info(&self, name: &str) -> Result<BackendPackageInfo, BackendError> {
        if !self.available {
            return Err(BackendError::NotAvailable(
                "APT is not available on this system".to_string(),
            ));
        }

        let result = Command::new("apt-cache")
            .args(&["show", name])
            .output()
            .await
            .map_err(|e| BackendError::Unknown(format!("Failed to get package info: {}", e)))?;

        if !result.status.success() {
            return Err(BackendError::PackageNotFound(format!(
                "Package '{}' not found",
                name
            )));
        }

        let output = String::from_utf8_lossy(&result.stdout);
        self.parse_apt_show(&output)
    }

    async fn install_package(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<BackendOperationResult, BackendError> {
        if !self.available {
            return Err(BackendError::NotAvailable(
                "APT is not available on this system".to_string(),
            ));
        }

        let package_spec = if let Some(ver) = version {
            format!("{}={}", name, ver)
        } else {
            name.to_string()
        };

        let result = self
            .run_apt_command(&["install", "-y", &package_spec])
            .await?;

        if !result.success {
            return Err(BackendError::InstallationFailed(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        Ok(result)
    }

    async fn remove_package(
        &self,
        name: &str,
        purge: bool,
    ) -> Result<BackendOperationResult, BackendError> {
        if !self.available {
            return Err(BackendError::NotAvailable(
                "APT is not available on this system".to_string(),
            ));
        }

        let action = if purge { "purge" } else { "remove" };
        let result = self.run_apt_command(&[action, "-y", name]).await?;

        if !result.success {
            return Err(BackendError::RemovalFailed(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        Ok(result)
    }

    async fn is_installed(&self, name: &str) -> Result<bool, BackendError> {
        if !self.available {
            return Err(BackendError::NotAvailable(
                "APT is not available on this system".to_string(),
            ));
        }

        let result = Command::new("dpkg")
            .args(&["-s", name])
            .output()
            .await
            .map_err(|e| BackendError::Unknown(format!("Failed to check status: {}", e)))?;

        if !result.status.success() {
            return Ok(false);
        }

        // Проверяем, что статус "install ok installed"
        let output = String::from_utf8_lossy(&result.stdout);
        Ok(output.contains("Status: install ok installed"))
    }

    async fn list_installed(&self) -> Result<Vec<BackendPackageInfo>, BackendError> {
        if !self.available {
            return Err(BackendError::NotAvailable(
                "APT is not available on this system".to_string(),
            ));
        }

        let result = Command::new("dpkg-query")
            .args(&["-W", "-f=${Package}\t${Version}\n"])
            .output()
            .await
            .map_err(|e| BackendError::Unknown(format!("Failed to list packages: {}", e)))?;

        if !result.status.success() {
            return Err(BackendError::Unknown("Failed to list packages".to_string()));
        }

        let output = String::from_utf8_lossy(&result.stdout);
        let packages: Vec<BackendPackageInfo> = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    Some(BackendPackageInfo::minimal(parts[0], parts[1]))
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    async fn list_upgradable(&self) -> Result<Vec<BackendPackageInfo>, BackendError> {
        if !self.available {
            return Err(BackendError::NotAvailable(
                "APT is not available on this system".to_string(),
            ));
        }

        let result = Command::new("apt")
            .args(&["list", "--upgradable"])
            .output()
            .await
            .map_err(|e| BackendError::Unknown(format!("Failed to list upgradable: {}", e)))?;

        if !result.status.success() {
            return Err(BackendError::Unknown(
                "Failed to list upgradable packages".to_string(),
            ));
        }

        let output = String::from_utf8_lossy(&result.stdout);
        let packages: Vec<BackendPackageInfo> = output
            .lines()
            .skip(1) // Пропускаем заголовок "Listing..."
            .filter_map(|line| {
                // Формат: "package/repo version arch [upgradable from: old_version]"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].split('/').next().unwrap_or(parts[0]);
                    Some(BackendPackageInfo::minimal(name, parts[1]))
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    async fn upgrade_package(&self, name: &str) -> Result<BackendOperationResult, BackendError> {
        if !self.available {
            return Err(BackendError::NotAvailable(
                "APT is not available on this system".to_string(),
            ));
        }

        let result = self
            .run_apt_command(&["install", "--only-upgrade", "-y", name])
            .await?;

        if !result.success {
            return Err(BackendError::Unknown(
                result
                    .error
                    .unwrap_or_else(|| "Failed to upgrade package".to_string()),
            ));
        }

        Ok(result)
    }

    async fn get_dependencies(&self, name: &str) -> Result<Vec<String>, BackendError> {
        let info = self.get_package_info(name).await?;
        Ok(info.dependencies)
    }

    async fn check_dependencies(&self, name: &str) -> Result<(), BackendError> {
        if !self.available {
            return Err(BackendError::NotAvailable(
                "APT is not available on this system".to_string(),
            ));
        }

        // apt-get с -s делает симуляцию без реальной установки
        let result = Command::new("apt-get")
            .args(&["install", "-s", name])
            .output()
            .await
            .map_err(|e| BackendError::Unknown(format!("Failed to check dependencies: {}", e)))?;

        if result.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr);
            Err(BackendError::DependencyConflict(stderr.to_string()))
        }
    }
}

// ============================================================================
// FFI Export для динамической загрузки
// ============================================================================

/// Создаёт новый экземпляр APT бэкенда
///
/// Эта функция экспортируется для динамической загрузки через libloading
#[no_mangle]
pub extern "C" fn create_backend() -> *mut dyn PackageBackend {
    let mut backend = AptBackend::new();
    // Инициализация должна быть синхронной для FFI, поэтому просто создаём
    Box::into_raw(Box::new(backend))
}

/// Освобождает память бэкенда
#[no_mangle]
pub extern "C" fn destroy_backend(ptr: *mut dyn PackageBackend) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_backend_creation() {
        let backend = AptBackend::new();
        assert_eq!(backend.name(), "apt");
    }

    #[tokio::test]
    async fn test_availability_check() {
        let backend = AptBackend::new();
        // Тест просто проверяет, что метод не падает
        let _available = backend.is_available().await;
    }
}
