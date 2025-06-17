// Copyright 2020 Ant Group. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0

//! Validator for RAFS format

use std::path::Path;

use anyhow::{Context, Result};
use rafs::metadata::{RafsMode, RafsSuper};
use serde::{Deserialize, Serialize};

use crate::tree::Tree;

pub struct Validator {
    sb: RafsSuper,
}

#[derive(Serialize, Deserialize, Default)]
pub struct BlobEntry {
    pub index: u32,
    pub id: String,
    pub chunk_count: u32,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct File {
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub size: u64,
    pub blob_indexes: Vec<u32>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct CheckResult {
    pub blob_entries: Vec<BlobEntry>,
    pub blobs: Vec<String>,
    pub files: Vec<File>,
}

impl Validator {
    pub fn new(bootstrap_path: &Path) -> Result<Self> {
        let sb = RafsSuper::load_from_metadata(bootstrap_path, RafsMode::Direct, true)?;

        Ok(Self { sb })
    }

    pub fn check(&mut self, verbosity: bool) -> Result<CheckResult> {
        let err = "failed to load bootstrap for validator";
        let tree = Tree::from_bootstrap(&self.sb, &mut ()).context(err)?;

        let blob_entries = self
            .sb
            .superblock
            .get_blob_infos()
            .iter()
            .map(|entry| BlobEntry {
                index: entry.blob_index(),
                id: entry.blob_id().to_string(),
                chunk_count: entry.chunk_count(),
                compressed_size: entry.compressed_size(),
                uncompressed_size: entry.uncompressed_size(),
            })
            .collect::<Vec<BlobEntry>>();

        let mut files = Vec::new();
        tree.iterate(&mut |node| {
            if verbosity {
                info!("{}", node);
            }
            let path = node.target().to_string_lossy().to_string();
            let file_type = node.file_type().to_string();
            let size = node.inode.size();
            let mut blob_indexes = node
                .chunks
                .iter()
                .map(|chunk| chunk.inner.blob_index())
                .collect::<Vec<u32>>();
            blob_indexes.sort();
            blob_indexes.dedup();
            files.push(File {
                path,
                file_type,
                size,
                blob_indexes,
            });
            true
        })?;

        let blobs = self
            .sb
            .superblock
            .get_blob_infos()
            .iter()
            .map(|entry| entry.blob_id().to_string())
            .collect::<Vec<String>>();

        Ok(CheckResult {
            blob_entries,
            blobs,
            files,
        })
    }
}
