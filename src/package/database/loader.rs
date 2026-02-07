use anyhow::{Context, Result};
use std::collections::HashMap;

use super::cache::{CacheMetadata, DatabaseCache};

/// URL to the package database
const DATABASE_URL: &str =
    "https://github.com/limistah/heimdal-packages/releases/latest/download/packages.db";
/// URL to the checksum file
const CHECKSUM_URL: &str =
    "https://github.com/limistah/heimdal-packages/releases/latest/download/packages.db.sha256";

/// Compiled package database structure (matches heimdal-packages compiler output)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CompiledDatabase {
    pub version: u32,
    pub last_updated: String,
    pub packages: Vec<Package>,
    pub groups: Vec<PackageGroup>,
    // Indexes for fast lookups
    pub index_by_name: HashMap<String, usize>,
    pub index_by_category: HashMap<String, Vec<usize>>,
    pub index_by_tag: HashMap<String, Vec<usize>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub category: String,
    pub popularity: u8,
    pub platforms: Platforms,
    #[serde(default)]
    pub dependencies: Dependencies,
    #[serde(default)]
    pub alternatives: Vec<String>,
    #[serde(default)]
    pub related: Vec<String>,
    pub tags: Vec<String>,
    pub website: Option<String>,
    pub license: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Platforms {
    pub apt: Option<String>,
    pub brew: Option<String>,
    pub dnf: Option<String>,
    pub pacman: Option<String>,
    pub mas: Option<i64>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Dependencies {
    #[serde(default)]
    pub required: Vec<Dependency>,
    #[serde(default)]
    pub optional: Vec<Dependency>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Dependency {
    pub package: String,
    pub reason: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PackageGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub packages: GroupPackages,
    #[serde(default)]
    pub platform_overrides: HashMap<String, PlatformOverride>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GroupPackages {
    pub required: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PlatformOverride {
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub casks: Vec<String>,
}

/// Loads the package database from cache or downloads it
pub struct DatabaseLoader {
    cache: DatabaseCache,
}

impl DatabaseLoader {
    /// Create a new database loader
    pub fn new() -> Result<Self> {
        let cache = DatabaseCache::new()?;
        Ok(Self { cache })
    }

    /// Load the database, downloading if necessary
    pub fn load(&self, force_update: bool) -> Result<CompiledDatabase> {
        // Check if we need to update
        if force_update || !self.cache.exists() {
            self.download()?;
        }

        // Verify checksum
        if !self.cache.verify_checksum()? {
            anyhow::bail!("Database checksum verification failed. Run 'heimdal packages update' to re-download.");
        }

        // Load and deserialize
        let data = self.cache.read_db()?;
        let db: CompiledDatabase =
            bincode::deserialize(&data).context("Failed to deserialize package database")?;

        Ok(db)
    }

    /// Load with auto-update check (updates if older than max_age_days)
    pub fn load_with_auto_update(&self, max_age_days: i64) -> Result<CompiledDatabase> {
        if self.cache.needs_update(max_age_days) {
            match self.download() {
                Ok(_) => {}
                Err(e) => {
                    // If download fails but we have a cached version, use it
                    if self.cache.exists() {
                        eprintln!("Warning: Failed to update package database: {}", e);
                        eprintln!("Using cached version (may be outdated)");
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        self.load(false)
    }

    /// Download the database from GitHub releases
    pub fn download(&self) -> Result<()> {
        use sha2::{Digest, Sha256};

        // Download checksum first
        let checksum_response =
            reqwest::blocking::get(CHECKSUM_URL).context("Failed to download database checksum")?;

        if !checksum_response.status().is_success() {
            anyhow::bail!(
                "Failed to download checksum: HTTP {}",
                checksum_response.status()
            );
        }

        let expected_checksum = checksum_response
            .text()
            .context("Failed to read checksum")?
            .trim()
            .to_lowercase();

        // Download database
        let db_response =
            reqwest::blocking::get(DATABASE_URL).context("Failed to download package database")?;

        if !db_response.status().is_success() {
            anyhow::bail!("Failed to download database: HTTP {}", db_response.status());
        }

        let db_data = db_response.bytes().context("Failed to read database")?;

        // Verify checksum
        let mut hasher = Sha256::new();
        hasher.update(&db_data);
        let computed_checksum = format!("{:x}", hasher.finalize());

        if computed_checksum != expected_checksum {
            anyhow::bail!(
                "Database checksum mismatch! Expected: {}, got: {}",
                expected_checksum,
                computed_checksum
            );
        }

        // Deserialize to get metadata
        let db: CompiledDatabase =
            bincode::deserialize(&db_data).context("Downloaded database is corrupted")?;

        // Save to cache
        self.cache.write_db(&db_data)?;
        self.cache.write_checksum(&expected_checksum)?;

        let metadata = CacheMetadata::new(db.version, db.packages.len(), db_data.len());
        self.cache.write_metadata(&metadata)?;

        Ok(())
    }

    /// Get cache metadata
    pub fn get_cache_info(&self) -> Result<Option<CacheMetadata>> {
        if !self.cache.exists() {
            return Ok(None);
        }
        Ok(Some(self.cache.read_metadata()?))
    }

    /// Clear the cache
    pub fn clear_cache(&self) -> Result<()> {
        self.cache.clear()
    }
}
