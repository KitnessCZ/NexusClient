
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

impl VersionManifest {
    pub fn parse(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|error| format!("Invalid Mojang version manifest: {error}"))
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

    #[test]
    fn parses_manifest() {
        let manifest = VersionManifest::parse(TEST_MANIFEST).unwrap();

        assert_eq!(manifest.latest.release, "26.3");
        assert_eq!(manifest.versions.len(), 1);
    }

    #[test]
    fn finds_minecraft_26_3() {
        let manifest = VersionManifest::parse(TEST_MANIFEST).unwrap();
        let version = manifest.target_release().unwrap();

        assert_eq!(version.id, "26.3");
        assert_eq!(version.version_type, "release");
        assert_eq!(version.url, "https://example.com/26.3.json");
    }

    #[test]
    fn rejects_missing_versions() {
        let manifest = VersionManifest::parse(TEST_MANIFEST).unwrap();

        assert!(manifest.find_version("1.8.9").is_err());
    }
}
