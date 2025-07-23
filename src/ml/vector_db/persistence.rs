/*! Vector Database Persistence
 * Efficient serialization and storage for vector databases
 */

use super::*;
use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Persistence manager for vector databases
pub struct VectorDBPersistence {
    base_path: PathBuf,
    compression_enabled: bool,
}

impl VectorDBPersistence {
    /// Create new persistence manager
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            compression_enabled: true,
        }
    }
    
    /// Enable/disable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.compression_enabled = enabled;
        self
    }
    
    /// Save vector database to disk
    pub fn save_database(&self, db: &dyn VectorDatabase) -> Result<()> {
        std::fs::create_dir_all(&self.base_path)?;
        
        // Save metadata
        let stats = db.stats();
        self.save_stats(&stats)?;
        
        // Save vectors in batches for better performance
        self.save_vectors_batched(db)?;
        
        Ok(())
    }
    
    /// Load vector database from disk
    pub fn load_database(&self, db: &mut dyn VectorDatabase) -> Result<()> {
        if !self.base_path.exists() {
            return Ok(()); // No data to load
        }
        
        // Load vectors
        self.load_vectors(db)?;
        
        Ok(())
    }
    
    /// Save database statistics
    fn save_stats(&self, stats: &VectorDBStats) -> Result<()> {
        let stats_path = self.base_path.join("stats.json");
        let file = File::create(stats_path)?;
        let writer = BufWriter::new(file);
        
        serde_json::to_writer_pretty(writer, stats)?;
        
        Ok(())
    }
    
    /// Save vectors in batches for memory efficiency
    fn save_vectors_batched(&self, db: &dyn VectorDatabase) -> Result<()> {
        let vectors_dir = self.base_path.join("vectors");
        std::fs::create_dir_all(&vectors_dir)?;
        
        // Get all vectors from the database
        let all_vectors = db.get_all_vectors()?;
        let mut batch_id = 0;
        
        // If no vectors, create empty batch index
        if all_vectors.is_empty() {
            let index_path = self.base_path.join("batch_index.json");
            let batch_info = BatchIndex {
                total_batches: 0,
                created_at: chrono::Utc::now(),
            };
            
            let file = File::create(index_path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &batch_info)?;
            return Ok(());
        }
        
        // Save in batches of reasonable size
        const BATCH_SIZE: usize = 1000;
        for chunk in all_vectors.chunks(BATCH_SIZE) {
            if !chunk.is_empty() {
                self.save_vector_batch(&vectors_dir, batch_id, chunk)?;
                batch_id += 1;
            }
        }
        
        // Save batch index
        let index_path = self.base_path.join("batch_index.json");
        let batch_info = BatchIndex {
            total_batches: batch_id,
            created_at: chrono::Utc::now(),
        };
        
        let file = File::create(index_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &batch_info)?;
        
        Ok(())
    }
    
    /// Save a batch of vectors
    fn save_vector_batch(
        &self,
        vectors_dir: &Path,
        batch_id: usize,
        vectors: &[VectorEntry],
    ) -> Result<()> {
        let batch_path = vectors_dir.join(format!("batch_{:06}.json", batch_id));
        let file = File::create(batch_path)?;
        let writer = BufWriter::new(file);
        
        let batch = VectorBatch {
            id: batch_id,
            vectors: vectors.to_vec(),
            created_at: chrono::Utc::now(),
        };
        
        if self.compression_enabled {
            // Use compact JSON without pretty printing for storage efficiency
            serde_json::to_writer(writer, &batch)?;
        } else {
            serde_json::to_writer_pretty(writer, &batch)?;
        }
        
        Ok(())
    }
    
    /// Load vectors from disk
    fn load_vectors(&self, db: &mut dyn VectorDatabase) -> Result<()> {
        let vectors_dir = self.base_path.join("vectors");
        if !vectors_dir.exists() {
            return Ok(());
        }
        
        // Load batch index
        let index_path = self.base_path.join("batch_index.json");
        if !index_path.exists() {
            return Ok(());
        }
        
        let index_file = File::open(index_path)?;
        let reader = BufReader::new(index_file);
        let batch_index: BatchIndex = serde_json::from_reader(reader)?;
        
        // Load all batches
        let mut total_loaded = 0;
        for batch_id in 0..batch_index.total_batches {
            let batch_path = vectors_dir.join(format!("batch_{:06}.json", batch_id));
            if batch_path.exists() {
                let loaded = self.load_vector_batch(db, &batch_path)?;
                total_loaded += loaded;
            }
        }
        
        tracing::info!("Loaded {} vectors from {} batches", total_loaded, batch_index.total_batches);
        
        Ok(())
    }
    
    /// Load a batch of vectors
    fn load_vector_batch(&self, db: &mut dyn VectorDatabase, batch_path: &Path) -> Result<usize> {
        let file = File::open(batch_path)?;
        let reader = BufReader::new(file);
        let batch: VectorBatch = serde_json::from_reader(reader)?;
        
        let count = batch.vectors.len();
        db.add_vectors(batch.vectors)?;
        
        Ok(count)
    }
    
    /// Export database to different formats
    pub fn export_database(&self, db: &dyn VectorDatabase, format: ExportFormat) -> Result<()> {
        match format {
            ExportFormat::Json => self.export_json(db),
            ExportFormat::Csv => self.export_csv(db),
            ExportFormat::Parquet => self.export_parquet(db),
        }
    }
    
    /// Export to JSON format
    fn export_json(&self, db: &dyn VectorDatabase) -> Result<()> {
        let export_path = self.base_path.join("export.json");
        let file = File::create(export_path)?;
        let writer = BufWriter::new(file);
        
        let stats = db.stats();
        let export_data = DatabaseExport {
            stats,
            format_version: "1.0".to_string(),
            exported_at: chrono::Utc::now(),
        };
        
        serde_json::to_writer_pretty(writer, &export_data)?;
        
        Ok(())
    }
    
    /// Export to CSV format (metadata only)
    fn export_csv(&self, _db: &dyn VectorDatabase) -> Result<()> {
        let export_path = self.base_path.join("export.csv");
        let mut file = File::create(export_path)?;
        
        // Write CSV header
        writeln!(file, "id,file_path,function_name,code_type,language,complexity,line_start,line_end")?;
        
        // TODO: Implement CSV export of vector metadata
        // This would require iterating through all vectors
        
        Ok(())
    }
    
    /// Export to Parquet format (for analytics)
    fn export_parquet(&self, _db: &dyn VectorDatabase) -> Result<()> {
        // TODO: Implement Parquet export using arrow-rs
        // This would be useful for data analysis workflows
        anyhow::bail!("Parquet export not yet implemented");
    }
    
    /// Cleanup old backup files
    pub fn cleanup_old_backups(&self, keep_days: u32) -> Result<()> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(keep_days as i64);
        
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified_dt = chrono::DateTime::<chrono::Utc>::from(modified);
                    if modified_dt < cutoff {
                        if path.is_file() {
                            std::fs::remove_file(path)?;
                        } else if path.is_dir() {
                            std::fs::remove_dir_all(path)?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Create incremental backup
    pub fn create_backup(&self, db: &dyn VectorDatabase, backup_name: &str) -> Result<()> {
        let backup_dir = self.base_path.join("backups").join(backup_name);
        std::fs::create_dir_all(&backup_dir)?;
        
        let backup_persistence = VectorDBPersistence::new(backup_dir);
        backup_persistence.save_database(db)?;
        
        // Create backup metadata
        let backup_info = BackupInfo {
            name: backup_name.to_string(),
            created_at: chrono::Utc::now(),
            stats: db.stats(),
        };
        
        let info_path = self.base_path.join("backups").join(format!("{}.info.json", backup_name));
        let file = File::create(info_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &backup_info)?;
        
        Ok(())
    }
    
    /// List available backups
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let backups_dir = self.base_path.join("backups");
        if !backups_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut backups = Vec::new();
        
        for entry in std::fs::read_dir(backups_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("json")) {
                if let Some(stem) = path.file_stem() {
                    if stem.to_string_lossy().ends_with(".info") {
                        let file = File::open(path)?;
                        let reader = BufReader::new(file);
                        if let Ok(backup_info) = serde_json::from_reader::<_, BackupInfo>(reader) {
                            backups.push(backup_info);
                        }
                    }
                }
            }
        }
        
        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(backups)
    }
}

/// Batch index for tracking vector batches
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BatchIndex {
    total_batches: usize,
    created_at: chrono::DateTime<chrono::Utc>,
}

/// Vector batch for efficient storage
#[derive(Clone, Debug, Serialize, Deserialize)]
struct VectorBatch {
    id: usize,
    vectors: Vec<VectorEntry>,
    created_at: chrono::DateTime<chrono::Utc>,
}

/// Database export wrapper
#[derive(Clone, Debug, Serialize, Deserialize)]
struct DatabaseExport {
    stats: VectorDBStats,
    format_version: String,
    exported_at: chrono::DateTime<chrono::Utc>,
}

/// Backup information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub stats: VectorDBStats,
}

/// Export formats
#[derive(Clone, Debug)]
pub enum ExportFormat {
    Json,
    Csv,
    Parquet,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::vector_db::{vector_store::NativeVectorStore, VectorDBConfig};
    use tempfile::TempDir;
    
    #[test]
    fn test_persistence_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = VectorDBPersistence::new(temp_dir.path());
        
        let config = VectorDBConfig::default();
        let mut store = NativeVectorStore::new(config);
        
        // Add test data
        let entry = VectorEntry {
            id: "test1".to_string(),
            embedding: vec![1.0; 768],
            metadata: CodeMetadata {
                file_path: "test.ts".to_string(),
                function_name: Some("testFunc".to_string()),
                line_start: 1,
                line_end: 10,
                code_type: CodeType::Function,
                language: "typescript".to_string(),
                complexity: 1.0,
                tokens: vec!["test".to_string()],
                hash: "hash123".to_string(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        store.add_vector(entry.clone()).unwrap();
        
        // Save to disk
        persistence.save_database(&store).unwrap();
        
        // Create new store and load
        let mut new_store = NativeVectorStore::new(VectorDBConfig::default());
        persistence.load_database(&mut new_store).unwrap();
        
        // Verify data was loaded
        let loaded_entry = new_store.get_by_id("test1").unwrap();
        assert!(loaded_entry.is_some());
        assert_eq!(loaded_entry.unwrap().id, "test1");
    }
    
    #[test]
    fn test_backup_operations() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = VectorDBPersistence::new(temp_dir.path());
        
        let config = VectorDBConfig::default();
        let mut store = NativeVectorStore::new(config);
        
        // Create backup
        persistence.create_backup(&store, "test_backup").unwrap();
        
        // List backups
        let backups = persistence.list_backups().unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].name, "test_backup");
    }
}