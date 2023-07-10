//! Structs that represent the response from the Simple API when using JSON (PEP 691).

use crate::artifact_name::ArtifactName;
use rattler_digest::{serde::SerializableHash, Md5, Sha256};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

/// Represents the result of the response from the Simple API.
#[serde_as]
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ProjectInfo {
    /// Metadata describing the API.
    pub meta: Meta,

    /// All the available files for this project
    pub files: Vec<ArtifactInfo>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ArtifactInfo {
    name: ArtifactName,
    url: url::Url,
    hashes: ArtifactHashes,
    requires_python: Option<String>,
    #[serde(default)]
    dist_info_metadata: DistInfoMetadata,
    #[serde(default)]
    yanked: Yanked,
}

/// Describes a set of hashes for a certain artifact. In theory all hash algorithms available via
/// Pythons `hashlib` are supported but we only support some common ones.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct ArtifactHashes {
    #[serde_as(as = "Option<SerializableHash<Sha256>>")]
    sha256: Option<rattler_digest::Sha256Hash>,

    #[serde_as(as = "Option<SerializableHash<Md5>>")]
    md5: Option<rattler_digest::Md5Hash>,
}

impl ArtifactHashes {
    /// Returns true if this instance does not contain a single hash.
    pub fn is_empty(&self) -> bool {
        self.sha256.is_none() && self.md5.is_none()
    }
}

/// Describes whether the metadata is available for download from the index as specified in PEP 658
/// (`{file_url}.metadata`). An index might also include hashes of the metadata file.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(from = "Option<RawDistInfoMetadata>")]
pub struct DistInfoMetadata {
    pub available: bool,
    pub hashes: ArtifactHashes,
}

/// An optional key that indicates that metadata for this file is available, via the same location
/// as specified in PEP 658 ({file_url}.metadata). Where this is present, it MUST be either a
/// boolean to indicate if the file has an associated metadata file, or a dictionary mapping hash
/// names to a hex encoded digest of the metadata’s hash.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawDistInfoMetadata {
    NoHashes(bool),
    WithHashes(ArtifactHashes),
}

impl From<Option<RawDistInfoMetadata>> for DistInfoMetadata {
    fn from(maybe_raw: Option<RawDistInfoMetadata>) -> Self {
        match maybe_raw {
            None => Default::default(),
            Some(raw) => match raw {
                RawDistInfoMetadata::NoHashes(available) => Self {
                    available,
                    hashes: Default::default(),
                },
                RawDistInfoMetadata::WithHashes(hashes) => Self {
                    available: true,
                    hashes,
                },
            },
        }
    }
}

/// Meta information stored in the [`ProjectInfo`]. It represents the version of the API. Clients
/// should verify that the contents is as expected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Meta {
    #[serde(rename = "api-version")]
    pub version: String,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            version: "1.0".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawYanked {
    NoReason(bool),
    WithReason(String),
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(from = "RawYanked")]
pub struct Yanked {
    pub yanked: bool,
    pub reason: Option<String>,
}

impl From<RawYanked> for Yanked {
    fn from(raw: RawYanked) -> Self {
        match raw {
            RawYanked::NoReason(yanked) => Self {
                yanked,
                reason: None,
            },
            RawYanked::WithReason(reason) => Self {
                yanked: true,
                reason: Some(reason),
            },
        }
    }
}