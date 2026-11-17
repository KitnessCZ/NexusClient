
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

pub const TARGET_VERSION: &str = "26.3";

#[derive(Debug, Clone, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionEntry {
    pub id: String,

    #[serde(rename = "type")]
    pub version_type: String,

    pub url: String,
    pub time: Option<String>,

    #[serde(rename = "releaseTime")]
    pub release_time: Option<String>,

    pub sha1: Option<String>,

    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionMetadata {
    pub id: String,
    pub downloads: VersionDownloads,

    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,

    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,

    #[serde(default)]
    pub libraries: Vec<LibraryEntry>,

    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionDownloads {
    pub client: DownloadInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,

    #[serde(rename = "totalSize")]
    pub total_size: Option<u64>,

    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JavaVersion {
    pub component: String,

    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryEntry {
    pub name: String,

    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,

    #[serde(default)]
    pub url: Option<String>,

    #[serde(default)]
    pub rules: Vec<serde_json::Value>,

    #[serde(default)]
    pub natives: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<DownloadInfo>,

    #[serde(default)]
    pub classifiers: HashMap<String, DownloadInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndexData {
    pub objects: HashMap<String, AssetObject>,

    #[serde(default, rename = "virtual")]
    pub virtual_assets: Option<bool>,

    #[serde(default, rename = "map_to_resources")]
    pub map_to_resources: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

impl AssetIndexData {
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    pub fn total_size(&self) -> u64 {
        self.objects.values().map(|object| object.size).sum()
    }
}

impl VersionMetadata {
    pub fn library_artifacts(&self) -> Vec<(&str, &DownloadInfo)> {
        self.libraries
            .iter()
            .filter_map(|library| {
                let artifact = library
                    .downloads
                    .as_ref()?
                    .artifact
                    .as_ref()?;

                Some((library.name.as_str(), artifact))
            })
            .collect()
    }

    pub fn library_count(&self) -> usize {
        self.libraries.len()
    }
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("NexusClient/0.1.0")
        .build()
        .map_err(|error| format!("HTTP client error: {error}"))
}

fn download_text(url: &str) -> Result<String, String> {
    let response = http_client()?
        .get(url)
        .send()
        .map_err(|error| format!("Download failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("HTTP error: {error}"))?;

    response
        .text()
        .map_err(|error| format!("Response read failed: {error}"))
}

fn verify_bytes(
    bytes: &[u8],
    expected_size: u64,
    expected_sha1: &str,
) -> Result<(), String> {
    if bytes.len() as u64 != expected_size {
        return Err(format!(
            "File size mismatch: expected {} bytes, received {} bytes",
            expected_size,
            bytes.len()
        ));
    }

    let actual_hash = format!("{:x}", Sha1::digest(bytes));

    if !actual_hash.eq_ignore_ascii_case(expected_sha1) {
        return Err(format!(
            "SHA-1 mismatch: expected {}, received {}",
            expected_sha1,
            actual_hash
        ));
    }

    Ok(())
}

impl VersionManifest {
    pub fn parse(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|error| format!("Invalid Mojang version manifest: {error}"))
    }

    pub fn download() -> Result<Self, String> {
        let body = download_text(VERSION_MANIFEST_URL)?;
        Self::parse(&body)
    }

    pub fn find_version(&self, version: &str) -> Result<&VersionEntry, String> {
        self.versions
            .iter()
            .find(|entry| entry.id == version)
            .ok_or_else(|| format!("Minecraft version {version} was not found"))
    }

    pub fn latest_release(&self) -> Result<&VersionEntry, String> {
        self.find_version(&self.latest.release)
    }

    pub fn target_release(&self) -> Result<&VersionEntry, String> {
        self.find_version(TARGET_VERSION)
    }
}

impl VersionEntry {
    pub fn download_metadata(&self) -> Result<VersionMetadata, String> {
        let body = download_text(&self.url)?;

        let metadata: VersionMetadata = serde_json::from_str(&body)
            .map_err(|error| {
                format!("Invalid metadata for {}: {error}", self.id)
            })?;

        if metadata.id != self.id {
            return Err(format!(
                "Version mismatch: expected {}, received {}",
                self.id, metadata.id
            ));
        }

        Ok(metadata)
    }
}

impl AssetIndex {
    pub fn download_and_parse(&self) -> Result<AssetIndexData, String> {
        let response = http_client()?
            .get(&self.url)
            .send()
            .map_err(|error| {
                format!("Asset index download failed: {error}")
            })?
            .error_for_status()
            .map_err(|error| {
                format!("Asset index HTTP error: {error}")
            })?;

        let bytes = response
            .bytes()
            .map_err(|error| {
                format!("Cannot read asset index: {error}")
            })?;

        verify_bytes(&bytes, self.size, &self.sha1)?;

        serde_json::from_slice(&bytes)
            .map_err(|error| {
                format!("Invalid asset index JSON: {error}")
            })
    }
}

impl DownloadInfo {
    pub fn download_to(&self, destination: &Path) -> Result<PathBuf, String> {
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));

        fs::create_dir_all(parent)
            .map_err(|error| {
                format!("Cannot create download directory: {error}")
            })?;

        let file_name = destination
            .file_name()
            .ok_or_else(|| "Invalid destination filename".to_string())?
            .to_string_lossy();

        let temporary_path =
            destination.with_file_name(format!("{file_name}.part"));

        if let Err(error) = self.download_and_verify(&temporary_path) {
            let _ = fs::remove_file(&temporary_path);
            return Err(error);
        }

        fs::rename(&temporary_path, destination)
            .map_err(|error| {
                let _ = fs::remove_file(&temporary_path);
                format!("Cannot finalize downloaded file: {error}")
            })?;

        Ok(destination.to_path_buf())
    }

    fn download_and_verify(
        &self,
        temporary_path: &Path,
    ) -> Result<(), String> {
        let mut response = http_client()?
            .get(&self.url)
            .send()
            .map_err(|error| format!("Download failed: {error}"))?
            .error_for_status()
            .map_err(|error| format!("Download HTTP error: {error}"))?;

        let mut file = File::create(temporary_path)
            .map_err(|error| {
                format!("Cannot create temporary file: {error}")
            })?;

        let mut hasher = Sha1::new();
        let mut total_bytes: u64 = 0;
        let mut buffer = [0u8; 64 * 1024];

        loop {
            let count = response
                .read(&mut buffer)
                .map_err(|error| {
                    format!("Download interrupted: {error}")
                })?;

            if count == 0 {
                break;
            }

            total_bytes += count as u64;
            hasher.update(&buffer[..count]);

            file.write_all(&buffer[..count])
                .map_err(|error| {
                    format!("Cannot write downloaded file: {error}")
                })?;
        }

        file.flush()
            .map_err(|error| {
                format!("Cannot flush downloaded file: {error}")
            })?;

        if total_bytes != self.size {
            return Err(format!(
                "File size mismatch: expected {} bytes, received {} bytes",
                self.size,
                total_bytes
            ));
        }

        let actual_hash = format!("{:x}", hasher.finalize());

        if !actual_hash.eq_ignore_ascii_case(&self.sha1) {
            return Err(format!(
                "SHA-1 mismatch: expected {}, received {}",
                self.sha1,
                actual_hash
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MANIFEST: &str = r#"
    {
        "latest": {
            "release": "26.3",
            "snapshot": "26.4-snapshot-1"
        },
        "versions": [
            {
                "id": "26.3",
                "type": "release",
                "url": "https://example.com/26.3.json"
            }
        ]
    }
    "#;

    const TEST_METADATA: &str = r#"
    {
        "id": "26.3",
        "downloads": {
            "client": {
                "sha1": "clienthash",
                "size": 1000,
                "url": "https://example.com/client.jar"
            }
        },
        "assetIndex": {
            "id": "34",
            "sha1": "assethash",
            "size": 2000,
            "totalSize": 3000,
            "url": "https://example.com/assets.json"
        },
        "javaVersion": {
            "component": "java-runtime-epsilon",
            "majorVersion": 25
        },
        "libraries": [
            {
                "name": "org.example:test-library:1.0",
                "downloads": {
                    "artifact": {
                        "sha1": "testhash",
                        "size": 500,
                        "url": "https://example.com/test-library.jar"
                    },
                    "classifiers": {
                        "natives-linux": {
                            "sha1": "nativehash",
                            "size": 100,
                            "url": "https://example.com/native.jar"
                        }
                    }
                }
            }
        ],
        "arguments": {}
    }
    "#;

    const TEST_ASSET_INDEX: &str = r#"
    {
        "objects": {
            "minecraft/lang/en_us.json": {
                "hash": "0123456789abcdef0123456789abcdef01234567",
                "size": 1234
            },
            "minecraft/sounds.json": {
                "hash": "abcdef0123456789abcdef0123456789abcdef01",
                "size": 5678
            }
        },
        "virtual": false,
        "map_to_resources": true
    }
    "#;

    #[test]
    fn parses_manifest() {
        let manifest = VersionManifest::parse(TEST_MANIFEST).unwrap();

        assert_eq!(manifest.latest.release, "26.3");
        assert_eq!(manifest.versions.len(), 1);
    }

    #[test]
    fn finds_target_version() {
        let manifest = VersionManifest::parse(TEST_MANIFEST).unwrap();
        let version = manifest.target_release().unwrap();

        assert_eq!(version.id, "26.3");
        assert_eq!(version.version_type, "release");
    }

    #[test]
    fn rejects_missing_versions() {
        let manifest = VersionManifest::parse(TEST_MANIFEST).unwrap();

        assert!(manifest.find_version("1.8.9").is_err());
    }

    #[test]
    fn parses_version_metadata() {
        let metadata: VersionMetadata =
            serde_json::from_str(TEST_METADATA).unwrap();

        assert_eq!(metadata.id, "26.3");
        assert_eq!(metadata.downloads.client.sha1, "clienthash");
        assert_eq!(metadata.downloads.client.size, 1000);
        assert_eq!(metadata.asset_index.id, "34");
        assert_eq!(metadata.java_version.major_version, 25);
    }

    #[test]
    fn parses_library_artifacts() {
        let metadata: VersionMetadata =
            serde_json::from_str(TEST_METADATA).unwrap();

        let artifacts = metadata.library_artifacts();

        assert_eq!(metadata.library_count(), 1);
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].0, "org.example:test-library:1.0");
        assert_eq!(artifacts[0].1.size, 500);
    }

    #[test]
    fn parses_library_classifiers() {
        let metadata: VersionMetadata =
            serde_json::from_str(TEST_METADATA).unwrap();

        let classifiers = metadata.libraries[0]
            .downloads
            .as_ref()
            .unwrap()
            .classifiers
            .get("natives-linux")
            .unwrap();

        assert_eq!(classifiers.size, 100);
        assert_eq!(classifiers.sha1, "nativehash");
    }

    #[test]
    fn parses_asset_index() {
        let index: AssetIndexData =
            serde_json::from_str(TEST_ASSET_INDEX).unwrap();

        assert_eq!(index.object_count(), 2);
        assert_eq!(index.total_size(), 6912);
        assert_eq!(index.virtual_assets, Some(false));
        assert_eq!(index.map_to_resources, Some(true));
    }

    #[test]
    fn verifies_asset_index_bytes() {
        let bytes = b"hello";

        let hash = format!("{:x}", Sha1::digest(bytes));

        assert!(verify_bytes(bytes, 5, &hash).is_ok());
        assert!(verify_bytes(bytes, 6, &hash).is_err());
        assert!(verify_bytes(bytes, 5, "wrong-hash").is_err());
    }
}
