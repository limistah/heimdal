use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Manages the local cache for the package database
pub struct DatabaseCache {
    cache_dir: PathBuf,
}

impl DatabaseCache {
    /// Database file name
    const DB_FILENAME: &'static str = "packages.db";
    /// Checksum file name
    const CHECKSUM_FILENAME: &'static str = "packages.db.sha256";
    /// Metadata file name
    const METADATA_FILENAME: &'static str = "packages.db.meta";

    /// Create a new cache manager
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// Get the cache directory path (~/.heimdal/cache)
    fn get_cache_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home_dir.join(".heimdal").join("cache"))
    }

    /// Get the path to the database file
    pub fn db_path(&self) -> PathBuf {
        self.cache_dir.join(Self::DB_FILENAME)
    }

    /// Get the path to the checksum file
    pub fn checksum_path(&self) -> PathBuf {
        self.cache_dir.join(Self::CHECKSUM_FILENAME)
    }

    /// Get the path to the metadata file
    pub fn metadata_path(&self) -> PathBuf {
        self.cache_dir.join(Self::METADATA_FILENAME)
    }

    /// Check if the database exists in cache
    pub fn exists(&self) -> bool {
        self.db_path().exists()
    }

    /// Read the database file
    pub fn read_db(&self) -> Result<Vec<u8>> {
        fs::read(self.db_path()).context("Failed to read cached database")
    }

    /// Write the database file
    pub fn write_db(&self, data: &[u8]) -> Result<()> {
        fs::write(self.db_path(), data).context("Failed to write database to cache")
    }

    /// Read the checksum file
    pub fn read_checksum(&self) -> Result<String> {
        fs::read_to_string(self.checksum_path()).context("Failed to read checksum")
    }

    /// Write the checksum file
    pub fn write_checksum(&self, checksum: &str) -> Result<()> {
        fs::write(self.checksum_path(), checksum).context("Failed to write checksum")
    }

    /// Read the metadata file (JSON with last_updated timestamp)
    pub fn read_metadata(&self) -> Result<CacheMetadata> {
        let content = fs::read_to_string(self.metadata_path())?;
        let metadata: CacheMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    /// Write the metadata file
    pub fn write_metadata(&self, metadata: &CacheMetadata) -> Result<()> {
        let content = serde_json::to_string_pretty(metadata)?;
        fs::write(self.metadata_path(), content)?;
        Ok(())
    }

    /// Get the age of the cache in days
    pub fn get_age_days(&self) -> Result<i64> {
        let metadata = self.read_metadata()?;
        let last_updated = chrono::DateTime::parse_from_rfc3339(&metadata.last_updated)?;
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(last_updated);
        Ok(duration.num_days())
    }

    /// Check if the cache needs updating (older than max_age_days)
    pub fn needs_update(&self, max_age_days: i64) -> bool {
        if !self.exists() {
            return true;
        }

        match self.get_age_days() {
            Ok(age) => age >= max_age_days,
            Err(_) => true, // If we can't read metadata, assume update needed
        }
    }

    /// Verify the checksum of the cached database
    pub fn verify_checksum(&self) -> Result<bool> {
        use sha2::{Digest, Sha256};

        let db_data = self.read_db()?;
        let stored_checksum = self.read_checksum()?.trim().to_lowercase();

        let mut hasher = Sha256::new();
        hasher.update(&db_data);
        let computed_checksum = format!("{:x}", hasher.finalize());

        Ok(computed_checksum == stored_checksum)
    }

    /// Clear the cache
    pub fn clear(&self) -> Result<()> {
        if self.db_path().exists() {
            fs::remove_file(self.db_path())?;
        }
        if self.checksum_path().exists() {
            fs::remove_file(self.checksum_path())?;
        }
        if self.metadata_path().exists() {
            fs::remove_file(self.metadata_path())?;
        }
        Ok(())
    }
}

/// Metadata about the cached database
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CacheMetadata {
    /// ISO 8601 timestamp of when the database was last updated
    pub last_updated: String,
    /// Version of the database
    pub version: u32,
    /// Number of packages in the database
    pub package_count: usize,
    /// Size in bytes
    pub size_bytes: usize,
}

impl CacheMetadata {
    pub fn new(version: u32, package_count: usize, size_bytes: usize) -> Self {
        Self {
            last_updated: chrono::Utc::now().to_rfc3339(),
            version,
            package_count,
            size_bytes,
        }
    }
}
