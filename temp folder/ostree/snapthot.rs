// ============================================================================
// Imports
// ============================================================================

use chrono::DateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Snapshot {
    uuid: Uuid,
    created: String,
    description: chrono::DateTime<chrono::Utc>,
}
