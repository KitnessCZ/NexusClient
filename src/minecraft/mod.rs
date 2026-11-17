
use serde::Deserialize;

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
    pub libraries: Vec<serde_json::Value>,

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

fn download_text(url: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("NexusClient/0.1.0")
        .build()
        .map_err(|error| format!("HTTP client error: {error}"))?;

    let response = client
        .get(url)
        .send()
        .map_err(|error| format!("Download failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("HTTP error: {error}"))?;

    response
        .text()
        .map_err(|error| format!("Response read failed: {error}"))
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
            .map_err(|error| format!("Invalid metadata for {}: {error}", self.id))?;

        if metadata.id != self.id {
            return Err(format!(
                "Version mismatch: expected {}, received {}",
                self.id, metadata.id
            ));
        }

        Ok(metadata)
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
                "url": "https://example.com/26.3.json",
                "time": "2026-09-15T00:00:00+00:00",
                "releaseTime": "2026-09-15T00:00:00+00:00",
                "sha1": "example",
                "complianceLevel": 1
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
        "libraries": [],
        "arguments": {}
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
        assert_eq!(
            metadata.java_version.component,
            "java-runtime-epsilon"
        );
    }
}
