use anyhow::Result;

use crate::model::{RunMetadata, TraceEvent};

pub fn encode_metadata(metadata: &RunMetadata) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(metadata)?)
}

pub fn encode_event(event: &TraceEvent) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(event)?)
}
