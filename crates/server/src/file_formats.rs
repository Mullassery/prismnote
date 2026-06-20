use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FileFormat {
    #[serde(rename = "parquet")]
    Parquet,
    #[serde(rename = "orc")]
    ORC,
    #[serde(rename = "avro")]
    Avro,
    #[serde(rename = "csv")]
    CSV,
    #[serde(rename = "json")]
    JSON,
    #[serde(rename = "delta")]
    DeltaLake,
    #[serde(rename = "iceberg")]
    Iceberg,
    #[serde(rename = "hudi")]
    Hudi,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileFormatInfo {
    pub format: String,
    pub description: String,
    pub compression_formats: Vec<String>,
    pub use_cases: Vec<String>,
    pub advantages: Vec<String>,
    pub disadvantages: Vec<String>,
    pub ecosystem_support: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParquetMetadata {
    pub version: u32,
    pub num_rows: u64,
    pub num_row_groups: u32,
    pub columns: Vec<ParquetColumnInfo>,
    pub compression_codecs: Vec<String>,
    pub created_by: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParquetColumnInfo {
    pub name: String,
    pub data_type: String,
    pub logical_type: Option<String>,
    pub num_values: u64,
    pub compression: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ORCMetadata {
    pub version: u32,
    pub num_rows: u64,
    pub num_stripes: u32,
    pub stripe_size: u64,
    pub compression: String,
    pub columns: Vec<ORCColumnInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ORCColumnInfo {
    pub name: String,
    pub data_type: String,
    pub encoded_size: u64,
    pub compressed_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeltaLakeMetadata {
    pub version: u64,
    pub num_rows: u64,
    pub num_files: u32,
    pub total_size_bytes: u64,
    pub schema: Vec<DeltaColumnInfo>,
    pub protocol_version: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeltaColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

pub struct FileFormatManager;

impl FileFormatManager {
    pub fn get_format_info(format: &str) -> Option<FileFormatInfo> {
        match format.to_lowercase().as_str() {
            "parquet" => Some(FileFormatInfo {
                format: "Parquet".to_string(),
                description: "Columnar storage format optimized for analytics".to_string(),
                compression_formats: vec![
                    "snappy".to_string(),
                    "gzip".to_string(),
                    "brotli".to_string(),
                    "zstd".to_string(),
                    "lz4".to_string(),
                ],
                use_cases: vec![
                    "Data warehousing".to_string(),
                    "OLAP queries".to_string(),
                    "Big data processing".to_string(),
                ],
                advantages: vec![
                    "Efficient compression".to_string(),
                    "Column-level operations".to_string(),
                    "Schema evolution support".to_string(),
                    "Wide ecosystem support".to_string(),
                ],
                disadvantages: vec![
                    "Not suitable for row-based access".to_string(),
                    "Larger files than uncompressed".to_string(),
                ],
                ecosystem_support: vec![
                    "Spark".to_string(),
                    "Flink".to_string(),
                    "Presto".to_string(),
                    "DuckDB".to_string(),
                    "Pandas".to_string(),
                ],
            }),
            "orc" => Some(FileFormatInfo {
                format: "ORC".to_string(),
                description: "Optimized Row Columnar format for Hadoop".to_string(),
                compression_formats: vec![
                    "zlib".to_string(),
                    "snappy".to_string(),
                    "lz4".to_string(),
                    "zstd".to_string(),
                ],
                use_cases: vec![
                    "Hive storage".to_string(),
                    "Hadoop ecosystem".to_string(),
                    "High compression".to_string(),
                ],
                advantages: vec![
                    "Excellent compression ratios".to_string(),
                    "Fast query processing".to_string(),
                    "Built-in indexes".to_string(),
                ],
                disadvantages: vec![
                    "Less mature than Parquet".to_string(),
                    "Limited tool support".to_string(),
                ],
                ecosystem_support: vec![
                    "Hive".to_string(),
                    "Spark".to_string(),
                    "Presto".to_string(),
                ],
            }),
            "avro" => Some(FileFormatInfo {
                format: "Avro".to_string(),
                description: "Row-oriented data format with schema".to_string(),
                compression_formats: vec![
                    "deflate".to_string(),
                    "snappy".to_string(),
                    "bzip2".to_string(),
                ],
                use_cases: vec![
                    "Messaging systems".to_string(),
                    "Stream processing".to_string(),
                    "Schema evolution".to_string(),
                ],
                advantages: vec![
                    "Compact encoding".to_string(),
                    "Schema included in data".to_string(),
                    "Good for streaming".to_string(),
                ],
                disadvantages: vec![
                    "Not columnar".to_string(),
                    "Larger than Parquet for analytics".to_string(),
                ],
                ecosystem_support: vec![
                    "Kafka".to_string(),
                    "Spark".to_string(),
                    "Flink".to_string(),
                ],
            }),
            "delta" => Some(FileFormatInfo {
                format: "Delta Lake".to_string(),
                description: "Storage layer providing ACID transactions over data lake".to_string(),
                compression_formats: vec![
                    "snappy".to_string(),
                    "gzip".to_string(),
                    "brotli".to_string(),
                    "zstd".to_string(),
                ],
                use_cases: vec![
                    "ACID transactions".to_string(),
                    "Data governance".to_string(),
                    "Time travel".to_string(),
                ],
                advantages: vec![
                    "ACID guarantees".to_string(),
                    "Schema enforcement".to_string(),
                    "Time travel queries".to_string(),
                    "DML support".to_string(),
                ],
                disadvantages: vec![
                    "Requires Databricks ecosystem".to_string(),
                    "Additional overhead".to_string(),
                ],
                ecosystem_support: vec![
                    "Spark".to_string(),
                    "Databricks".to_string(),
                    "Presto".to_string(),
                    "Flink".to_string(),
                ],
            }),
            "iceberg" => Some(FileFormatInfo {
                format: "Apache Iceberg".to_string(),
                description: "Open table format for huge analytic tables".to_string(),
                compression_formats: vec![
                    "snappy".to_string(),
                    "gzip".to_string(),
                    "brotli".to_string(),
                    "zstd".to_string(),
                ],
                use_cases: vec![
                    "Large-scale data".to_string(),
                    "Table formats".to_string(),
                    "Schema evolution".to_string(),
                ],
                advantages: vec![
                    "Schema evolution".to_string(),
                    "Hidden partitioning".to_string(),
                    "Partition evolution".to_string(),
                    "Concurrent writes".to_string(),
                ],
                disadvantages: vec![
                    "Metadata overhead".to_string(),
                    "Learning curve".to_string(),
                ],
                ecosystem_support: vec![
                    "Spark".to_string(),
                    "Flink".to_string(),
                    "Presto".to_string(),
                    "DuckDB".to_string(),
                    "Trino".to_string(),
                ],
            }),
            "hudi" => Some(FileFormatInfo {
                format: "Apache Hudi".to_string(),
                description: "Data lake framework for incremental processing".to_string(),
                compression_formats: vec![
                    "snappy".to_string(),
                    "gzip".to_string(),
                ],
                use_cases: vec![
                    "Incremental ingestion".to_string(),
                    "Near real-time".to_string(),
                    "CDC support".to_string(),
                ],
                advantages: vec![
                    "Incremental processing".to_string(),
                    "CDC support".to_string(),
                    "Rollback capability".to_string(),
                ],
                disadvantages: vec![
                    "Complex operations".to_string(),
                    "Metadata overhead".to_string(),
                ],
                ecosystem_support: vec![
                    "Spark".to_string(),
                    "Flink".to_string(),
                    "Presto".to_string(),
                ],
            }),
            _ => None,
        }
    }

    pub fn list_all_formats() -> Vec<FileFormatInfo> {
        vec![
            Self::get_format_info("parquet").unwrap(),
            Self::get_format_info("orc").unwrap(),
            Self::get_format_info("avro").unwrap(),
            Self::get_format_info("delta").unwrap(),
            Self::get_format_info("iceberg").unwrap(),
            Self::get_format_info("hudi").unwrap(),
        ]
    }

    pub async fn read_parquet_metadata(file_path: &str) -> Result<ParquetMetadata, String> {
        Ok(ParquetMetadata {
            version: 1,
            num_rows: 0,
            num_row_groups: 0,
            columns: vec![],
            compression_codecs: vec![],
            created_by: None,
        })
    }

    pub async fn read_orc_metadata(file_path: &str) -> Result<ORCMetadata, String> {
        Ok(ORCMetadata {
            version: 1,
            num_rows: 0,
            num_stripes: 0,
            stripe_size: 0,
            compression: "ZLIB".to_string(),
            columns: vec![],
        })
    }

    pub async fn read_delta_metadata(table_path: &str) -> Result<DeltaLakeMetadata, String> {
        Ok(DeltaLakeMetadata {
            version: 0,
            num_rows: 0,
            num_files: 0,
            total_size_bytes: 0,
            schema: vec![],
            protocol_version: 1,
        })
    }

    pub fn get_recommended_format(use_case: &str) -> String {
        match use_case.to_lowercase().as_str() {
            "analytics" | "olap" | "data_warehouse" => "parquet".to_string(),
            "streaming" | "messaging" => "avro".to_string(),
            "transactions" | "acid" => "delta".to_string(),
            "large_scale" | "scale" => "iceberg".to_string(),
            "incremental" | "cdc" | "change_data_capture" => "hudi".to_string(),
            "hadoop" => "orc".to_string(),
            _ => "parquet".to_string(), // Default recommendation
        }
    }
}
