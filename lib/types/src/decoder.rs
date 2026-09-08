// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Read;

use upac_abi::error::ErrorKind;
use upac_abi::hook::CancelToken;
use upac_abi::request::CDecodeRequest;
use upac_abi::response::CDecodeResponse;
use upac_abi::types::{COwned, CSlice};
use upac_macro::{CTryToRust, RedbCodec, RustToC};

use super::error::DecodeError;
use super::package::{PackageDependency, PackageMeta};

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct DecodeRequest {
    pub package_path: String,
    pub output_dir: String,
    pub checksum: [u8; 32],
    pub cancel_token: *mut CancelToken,
}

#[derive(Debug, Clone, CTryToRust)]
pub struct DecodeResponse {
    pub meta: PackageMeta,
    pub dependencies: Vec<PackageDependency>,
    pub declarative_triggers: Vec<String>,
}

#[derive(Debug, Clone, RedbCodec)]
pub struct DeclarativeTrigger {
    pub format: String,
    pub triggers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoderTrigger {
    PreInstall,
    PostInstall,
    PreUpgrade,
    PostUpgrade,
    PreRemove,
    PostRemove,
}

impl DecoderTrigger {
    pub const ALL: [DecoderTrigger; 6] = [
        DecoderTrigger::PreInstall,
        DecoderTrigger::PostInstall,
        DecoderTrigger::PreUpgrade,
        DecoderTrigger::PostUpgrade,
        DecoderTrigger::PreRemove,
        DecoderTrigger::PostRemove,
    ];
}

pub fn parse_constraint_prefix(token: &[u8], operators: &[(&[u8], u8)]) -> Option<(u8, usize)> {
    operators
        .iter()
        .find(|(operator, _)| token.starts_with(operator))
        .map(|(operator, constraint)| (*constraint, operator.len()))
}

pub fn read_to_string<R: Read>(reader: &mut R) -> Result<String, DecodeError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    String::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
}
