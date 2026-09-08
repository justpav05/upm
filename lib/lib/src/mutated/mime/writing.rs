// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;
use std::process::Command;

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use super::{MimeError, PendingWrites, TotalWrites};

use crate::errors::CommonError;
use crate::fs::WrittenFile;
use crate::layout::mime::{APPLICATIONS_DIR, MIME_DB_DIR, UPDATE_DESKTOP_DATABASE_BIN, UPDATE_MIME_DATABASE_BIN};
use crate::orchestrator::context::{Context, ctx_get, ctx_take};
use crate::orchestrator::stage::{RollbackGuard, Stage, StageResult};

pub struct WritingStage;

impl Stage<MimeError> for WritingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), MimeError> {
        let mut pending = ctx_take!(context, PendingWrites);

        let total = ctx_get!(context, TotalWrites);

        let (path, content) = pending.0.pop_front().ok_or(CommonError::MissingResult)?;

        let written_file = WrittenFile::write(Path::new(path), content.as_bytes())?;

        let remaining = pending.0.len() as u64;
        let processed = total.0 - remaining;
        progress = progress.subject(path.to_owned()).progress(processed, total.0);

        let result = if pending.0.is_empty() {
            let _ = Command::new(UPDATE_MIME_DATABASE_BIN).arg(MIME_DB_DIR).status();
            let _ = Command::new(UPDATE_DESKTOP_DATABASE_BIN).arg(APPLICATIONS_DIR).status();

            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(pending);

        Ok((progress, result, Box::new(vec![written_file])))
    }
}
