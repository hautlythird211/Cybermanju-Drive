use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::errors::ResolutionError;
use crate::shard::*;

/// Memory-mapped shard reader for zero-copy access.
pub struct ShardReader {
    mmap: memmap2::Mmap,
    index: ShardIndex,
    header: ShardHeader,
    content_map: ContentMap,
}

impl ShardReader {
    /// Open a shard file with memory mapping.
    pub fn open(path: &Path) -> Result<Self, ResolutionError> {
        let file =
            File::open(path).map_err(|e| ResolutionError::IoError(format!("open shard: {}", e)))?;
        let mmap = unsafe {
            memmap2::Mmap::map(&file)
                .map_err(|e| ResolutionError::IoError(format!("mmap shard: {}", e)))?
        };

        // Parse the binary layout: header_len(4) + header_json + index_len(4) + index_json + ...
        let mut pos = 0;

        // Header
        if mmap.len() < 4 {
            return Err(ResolutionError::IoError("shard too short".into()));
        }
        let header_len = u32::from_le_bytes([mmap[0], mmap[1], mmap[2], mmap[3]]) as usize;
        pos += 4;
        if pos + header_len > mmap.len() {
            return Err(ResolutionError::IoError("header overflow".into()));
        }
        let header: ShardHeader = serde_json::from_slice(&mmap[pos..pos + header_len])
            .map_err(|e| ResolutionError::SerializationError(e.to_string()))?;
        pos += header_len;

        // Index
        if pos + 4 > mmap.len() {
            return Err(ResolutionError::IoError("index length missing".into()));
        }
        let index_len =
            u32::from_le_bytes([mmap[pos], mmap[pos + 1], mmap[pos + 2], mmap[pos + 3]]) as usize;
        pos += 4;
        if pos + index_len > mmap.len() {
            return Err(ResolutionError::IoError("index overflow".into()));
        }
        let index: ShardIndex = serde_json::from_slice(&mmap[pos..pos + index_len])
            .map_err(|e| ResolutionError::SerializationError(e.to_string()))?;
        pos += index_len;

        // Content map
        if pos + 4 > mmap.len() {
            return Err(ResolutionError::IoError(
                "content map length missing".into(),
            ));
        }
        let cm_len =
            u32::from_le_bytes([mmap[pos], mmap[pos + 1], mmap[pos + 2], mmap[pos + 3]]) as usize;
        pos += 4;
        if pos + cm_len > mmap.len() {
            return Err(ResolutionError::IoError("content map overflow".into()));
        }
        let content_map: ContentMap = serde_json::from_slice(&mmap[pos..pos + cm_len])
            .map_err(|e| ResolutionError::SerializationError(e.to_string()))?;

        Ok(Self {
            mmap,
            index,
            header,
            content_map,
        })
    }

    /// Get the shard header.
    pub fn header(&self) -> &ShardHeader {
        &self.header
    }

    /// Get the shard index.
    pub fn index(&self) -> &ShardIndex {
        &self.index
    }

    /// Get the content map.
    pub fn content_map(&self) -> &ContentMap {
        &self.content_map
    }

    /// Zero-copy read: returns a slice into the mmap for the requested resolution.
    pub fn read_resolution_raw(&self, file_id: &str, res: &str) -> Result<&[u8], ResolutionError> {
        let entry = self
            .index
            .files
            .get(file_id)
            .ok_or_else(|| ResolutionError::FileNotFound(file_id.to_string()))?;
        let level = entry
            .resolutions
            .get(res)
            .ok_or_else(|| ResolutionError::ResolutionNotFound(res.to_string()))?;

        let content_start = self.content_start_offset();
        let start = content_start + level.content_offset as usize;
        let end = start + level.content_length as usize;

        if end > self.mmap.len() {
            return Err(ResolutionError::IoError(
                "content offset out of bounds".into(),
            ));
        }

        Ok(&self.mmap[start..end])
    }

    /// List all file IDs in the shard.
    pub fn list_files(&self) -> Vec<&str> {
        self.index.files.keys().map(|s| s.as_str()).collect()
    }

    /// Get resolution info for a file.
    pub fn get_resolution_info(&self, file_id: &str, res: &str) -> Option<&ResolutionLevel> {
        self.index.files.get(file_id)?.resolutions.get(res)
    }

    /// Byte offset where content blobs begin in the mmap.
    fn content_start_offset(&self) -> usize {
        let mut pos = 0;
        // header_len(4) + header_json
        pos += 4;
        if pos + 4 <= self.mmap.len() {
            let hl = u32::from_le_bytes([
                self.mmap[pos],
                self.mmap[pos + 1],
                self.mmap[pos + 2],
                self.mmap[pos + 3],
            ]) as usize;
            pos += 4 + hl;
        }
        // index_len(4) + index_json
        if pos + 4 <= self.mmap.len() {
            let il = u32::from_le_bytes([
                self.mmap[pos],
                self.mmap[pos + 1],
                self.mmap[pos + 2],
                self.mmap[pos + 3],
            ]) as usize;
            pos += 4 + il;
        }
        // content_map_len(4) + content_map_json
        if pos + 4 <= self.mmap.len() {
            let cl = u32::from_le_bytes([
                self.mmap[pos],
                self.mmap[pos + 1],
                self.mmap[pos + 2],
                self.mmap[pos + 3],
            ]) as usize;
            pos += 4 + cl;
        }
        // erasure_meta_len(4) + erasure_meta_json
        if pos + 4 <= self.mmap.len() {
            let el = u32::from_le_bytes([
                self.mmap[pos],
                self.mmap[pos + 1],
                self.mmap[pos + 2],
                self.mmap[pos + 3],
            ]) as usize;
            pos += 4 + el;
        }
        pos
    }
}
