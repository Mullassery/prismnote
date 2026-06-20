# External Connections & Data Format Support

**Version:** v0.4
**Date:** June 20, 2026
**Status:** Implementation Complete

---

## Overview

PrismNote now provides comprehensive visibility and management of external connections (databases, data warehouses, file storage, DuckDB, and Apache Iceberg) plus support for modern open-source table and file formats.

---

## Part 1: External Connection Monitoring

### Features

#### Connection Status Dashboard
- **Real-time overview** with connected/disconnected/error counts
- **Individual connection cards** showing:
  - Connection name and provider
  - Current status with status icon
  - Latency measurement (milliseconds)
  - Queries run count
  - Data transferred (bytes)
  - Last health check time
- **Auto-refresh** with configurable intervals (default: 30s)
- **Health check** on-demand for any connection
- **Error messages** displayed when connections fail
- **Statistics panel** showing:
  - Active queries
  - Total uptime
  - Data transferred over lifetime

#### Supported Connection Types

1. **Database Connections**
   - PostgreSQL
   - MySQL/MariaDB
   - SQLite
   - Oracle
   - SQL Server
   - Status: Live connection monitoring
   - Latency tracking per query

2. **Data Warehouse Connections**
   - Snowflake
   - BigQuery
   - Amazon Redshift
   - Azure Synapse
   - Databricks
   - Status: Multi-cluster support
   - Cost tracking integration

3. **File Storage Mounts**
   - Amazon S3
   - Google Cloud Storage
   - Azure Blob Storage
   - Google Drive
   - HDFS
   - Status: Mount health checks
   - Access verification

4. **DuckDB (In-Process)**
   - Embedded analytics engine
   - Parquet/ORC file support
   - Iceberg table scanning
   - Status: Always available
   - Memory usage tracking

5. **Apache Iceberg Tables**
   - Warehouse-scale analytics
   - Multi-catalog support
   - Status: Catalog health checks
   - Snapshot tracking

### API Endpoints

```
GET /api/connections/overview
  Returns: ConnectionsOverview with all connection statuses

GET /api/connections/{id}
  Returns: Detailed connection information

POST /api/connections/{id}/health-check
  Returns: ConnectionHealthCheck with latency and status

GET /api/connections?type={type}
  Returns: Connections filtered by type

DELETE /api/connections/{id}
  Removes connection from monitoring
```

### Connection Data Structure

```rust
pub struct ExternalConnection {
    pub id: String,                              // UUID
    pub name: String,                            // User-friendly name
    pub connection_type: String,                 // database, data_warehouse, etc.
    pub provider: String,                        // postgres, snowflake, s3, etc.
    pub status: String,                          // connected, disconnected, error
    pub connected_at: Option<String>,            // RFC3339 timestamp
    pub last_checked: String,                    // Last health check time
    pub latency_ms: Option<u32>,                 // Response time in ms
    pub error_message: Option<String>,           // Error details if failed
    pub config: HashMap<String, String>,         // Provider-specific config
    pub stats: ConnectionStats,                  // Performance statistics
}

pub struct ConnectionStats {
    pub total_connections: u32,
    pub active_queries: u32,
    pub queries_run: u32,
    pub total_bytes_transferred: u64,
    pub uptime_seconds: u64,
}
```

---

## Part 2: DuckDB Integration

### What is DuckDB?

DuckDB is an in-process SQL OLAP database management system:
- Embedded library (not a server)
- Optimized for analytical queries
- Native Parquet and Iceberg support
- Multi-threaded execution
- Zero configuration

### Features

#### Query Execution
```sql
-- Direct SQL queries
SELECT * FROM my_table;

-- Parquet analysis
SELECT COUNT(*) FROM 'data.parquet';

-- Cross-format queries
SELECT * FROM table UNION ALL SELECT * FROM 'parquet_file.parquet';

-- Iceberg tables
SELECT * FROM iceberg_scan('s3://bucket/warehouse/table');
```

#### Table Management
- **Load Parquet files** - Upload .parquet files into DuckDB tables
- **Export to Parquet** - Save tables as Parquet files
- **Export to CSV** - Generate CSV exports
- **Schema introspection** - View table structure and statistics
- **Row counting** - Quick table size assessment

#### Extensions
DuckDB comes with pre-loaded extensions:
- **json** - JSON data parsing and functions
- **parquet** - Parquet file format support
- **httpfs** - HTTP(S) file system access
- **iceberg** - Apache Iceberg table format support

### API Endpoints

```
GET /api/duckdb/tables
  Returns: Vec<TableSchema> with all tables

GET /api/duckdb/tables/{name}
  Returns: TableSchema with details

POST /api/duckdb/query
  Body: { "query": "SELECT ..." }
  Returns: DuckDBQueryResult

POST /api/duckdb/load-parquet
  Body: Multipart file upload
  Returns: Table information

GET /api/duckdb/export-parquet/{table}
  Returns: File download (parquet)

GET /api/duckdb/extensions
  Returns: List of loaded extensions

POST /api/duckdb/query-table/{table}
  Returns: Table preview (100 rows)
```

### Usage Examples

#### Load and Query Parquet
```
1. Upload sales_data.parquet via UI
2. DuckDB creates 'sales_data' table
3. Query: SELECT * FROM sales_data LIMIT 10
4. Results displayed in explorer
```

#### Cross-Format Analysis
```
1. Load CSV: CREATE TABLE csv_data AS SELECT * FROM 'data.csv'
2. Load Parquet: CREATE TABLE parquet_data AS SELECT * FROM 'data.parquet'
3. Join: SELECT * FROM csv_data JOIN parquet_data ON csv_data.id = parquet_data.id
```

#### Iceberg Integration
```
1. Connect to S3 warehouse: s3://bucket/warehouse
2. Query: SELECT * FROM iceberg_scan('s3://bucket/warehouse/my_table')
3. Time travel: SELECT * FROM iceberg_scan('s3://bucket/warehouse/my_table') AS OF VERSION 3
```

---

## Part 3: Apache Iceberg Support

### What is Apache Iceberg?

Iceberg is an open table format for huge analytic tables:
- ACID transactions on data lakes
- Schema and partition evolution
- Concurrent writes
- Time-travel queries
- Hidden partitioning

### Features

#### Table Management
```rust
pub struct IcebergTable {
    pub table_id: String,
    pub database: String,
    pub table_name: String,
    pub location: String,
    pub format: String,                 // Parquet or ORC
    pub schema: Vec<IcebergColumn>,
    pub row_count: u64,
    pub file_count: u32,
    pub size_bytes: u64,
    pub created_at: String,
    pub snapshots: Vec<IcebergSnapshot>,
}
```

#### Snapshots & Time Travel
Each write creates a snapshot:
- Immutable version of table state
- Enables point-in-time queries
- Automatic retention policies
- Manual snapshot management

#### Operations Supported
- **Write** - Append new data (creates snapshot)
- **Replace** - Full table replacement
- **Delete** - Row-level deletes
- **Update** - Row-level updates
- **Compact** - Consolidate small files
- **Rewrite** - Optimize files

### API Endpoints

```
GET /api/iceberg/catalogs
  Returns: List of configured catalogs

GET /api/iceberg/tables
  Returns: All Iceberg tables

POST /api/iceberg/tables
  Body: { "database": "...", "table_name": "...", "schema": [...] }
  Returns: Created table info

GET /api/iceberg/tables/{id}/snapshots
  Returns: Historical snapshots

POST /api/iceberg/tables/{id}/time-travel
  Body: { "snapshot_id": "..." }
  Returns: Table state at snapshot

POST /api/iceberg/tables/{id}/compact
  Returns: Compaction status

DELETE /api/iceberg/tables/{id}/snapshots
  Body: { "older_than_days": 30 }
  Returns: Cleanup summary
```

### Catalog Types

1. **Hive Metastore** (default)
   - File-based metadata
   - Local or Hadoop HDFS
   - Simple setup

2. **AWS Glue**
   - AWS native metadata service
   - IAM authentication
   - Multi-account support

3. **JDBC**
   - Database-backed metadata
   - Any SQL database
   - Enterprise support

4. **REST**
   - REST API metadata service
   - Custom implementations
   - Microservice architecture

---

## Part 4: File Format Support

### Supported Formats

#### 1. Parquet
**Best for:** Analytics, columnar storage
```
- Compression: Snappy, Gzip, Brotli, Zstd, LZ4
- Size: Highly compressed
- Tools: Spark, Flink, Presto, DuckDB, Pandas
- Advantages: Efficient, wide ecosystem, schema evolution
```

#### 2. ORC (Optimized Row Columnar)
**Best for:** Hadoop ecosystem, Hive
```
- Compression: ZLIB, Snappy, LZ4, Zstd
- Size: Excellent compression
- Tools: Hive, Spark, Presto
- Advantages: High compression, built-in indexes
```

#### 3. Avro
**Best for:** Messaging, streaming
```
- Compression: Deflate, Snappy, Bzip2
- Size: Compact for serialization
- Tools: Kafka, Spark, Flink
- Advantages: Schema included, streaming-friendly
```

#### 4. Delta Lake
**Best for:** ACID transactions, data lake
```
- Compression: Snappy, Gzip, Brotli, Zstd
- Base: Parquet + transaction log
- Tools: Spark, Databricks, Presto, Flink
- Advantages: ACID, time travel, DML operations
```

#### 5. Apache Iceberg
**Best for:** Large-scale analytics
```
- Compression: Snappy, Gzip, Brotli, Zstd
- Base: Parquet + metadata
- Tools: Spark, Flink, Presto, DuckDB, Trino
- Advantages: Schema evolution, hidden partitioning
```

#### 6. Apache Hudi
**Best for:** Incremental ingestion
```
- Compression: Snappy, Gzip
- Base: Parquet + indexing
- Tools: Spark, Flink, Presto
- Advantages: CDC support, incremental processing
```

### Format Comparison Matrix

| Feature | Parquet | ORC | Avro | Delta | Iceberg | Hudi |
|---------|---------|-----|------|-------|---------|------|
| Columnar | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ |
| ACID | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ |
| Time Travel | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ |
| Schema Evolution | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Concurrent Writes | ✗ | ✗ | ✗ | Limited | ✓ | ✓ |
| CDC Support | ✗ | ✗ | ✗ | ✗ | Limited | ✓ |
| Hidden Partitioning | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |

### Format Recommendations

```rust
match use_case {
    "analytics" | "olap" => "parquet",
    "streaming" | "messaging" => "avro",
    "transactions" | "acid" => "delta",
    "large_scale" => "iceberg",
    "incremental" | "cdc" => "hudi",
    "hadoop" => "orc",
}
```

---

## UI Components

### 1. Connection Status Dashboard
**Location:** `components/ConnectionStatus.tsx`

Features:
- Overview cards (total, connected, disconnected, errors)
- Sortable/filterable connection list
- Real-time status updates
- Health check on-demand
- Expandable details per connection
- Auto-refresh toggle
- Error message display

### 2. DuckDB Explorer
**Location:** `components/DuckDBExplorer.tsx`

Features:
- Table browser with statistics
- Query editor with syntax highlighting
- Result viewer (100 rows default)
- Parquet file upload
- Export to Parquet/CSV
- Extension status
- Execution time tracking

### 3. Data Formats Reference
**Location:** `components/DataFormats.tsx`

Features:
- Format information display
- Compression codec details
- Use case recommendations
- Ecosystem support matrix
- Advantages/disadvantages
- Iceberg table explorer
- Format comparison

---

## Integration Examples

### Example 1: Analyze Parquet Data with DuckDB
```
1. Upload sales.parquet via DuckDBExplorer
2. Auto-creates 'sales' table
3. Query: SELECT region, SUM(amount) FROM sales GROUP BY region
4. Export results back to Parquet
```

### Example 2: Monitor Data Warehouse Connection
```
1. Add Snowflake connection in ConnectionStatus
2. Auto-monitors connection every 30s
3. View latency and query counts
4. Click "Check" button for health status
5. View error details if disconnection occurs
```

### Example 3: Time-Travel Query on Iceberg
```
1. Browse Iceberg tables in DataFormats component
2. See snapshot history
3. Travel to specific snapshot version
4. Query historical data
5. Compare snapshots for data changes
```

---

## Performance Considerations

### DuckDB
- Memory: Configurable limit (default: 4GB)
- Threads: Auto-detected CPU count
- Compression: Configurable per format
- Extensions: Load-on-demand

### Iceberg
- Metadata: Cached for performance
- Snapshots: Automatic cleanup (configurable)
- Files: Can be compacted for optimization
- Partition Pruning: Automatic

### Monitoring
- Connection latency: Measured per health check
- Query execution: Tracked per operation
- Data transfer: Cumulative bytes tracked
- Error rates: Counted per connection type

---

## Best Practices

### For DuckDB
1. Use Parquet for analytics instead of CSV
2. Enable compression (snappy for speed, zstd for ratio)
3. Export results as Parquet for sharing
4. Use Iceberg for large datasets (>100GB)

### For Iceberg
1. Design schema with evolution in mind
2. Use hidden partitioning (don't create partition columns)
3. Enable automatic snapshot cleanup
4. Monitor file count (compact if > 1000 files)

### For Connection Monitoring
1. Set appropriate health check intervals
2. Monitor latency trends (increase = issues)
3. Alert on connection errors (3+ consecutive fails)
4. Regular backup of connection configs

---

## Troubleshooting

### DuckDB Connection Errors
- Verify file paths are accessible
- Check file formats are supported (parquet, orc, avro)
- Ensure sufficient disk space for queries

### Iceberg Issues
- Verify catalog URI is reachable
- Check S3/cloud storage permissions
- Ensure metadata files aren't corrupted

### Connection Status Problems
- Increase health check timeout for slow networks
- Verify firewall rules allow connections
- Check credentials are still valid

---

## Future Enhancements

- **Benchmarking suite** for format performance comparison
- **Format conversion** tool (Parquet ↔ Iceberg ↔ Delta)
- **Advanced Iceberg operations** (branch, tag, partition evolution)
- **Connection pooling** for high-concurrency scenarios
- **Cost estimation** for cloud warehouse queries
- **Data lineage tracking** across connections

---

## References

- [DuckDB Documentation](https://duckdb.org)
- [Apache Iceberg Spec](https://iceberg.apache.org)
- [Parquet Format](https://parquet.apache.org)
- [Delta Lake Documentation](https://delta.io)
- [Apache Hudi](https://hudi.apache.org)

---

This comprehensive integration provides PrismNote users with visibility into their entire data infrastructure and support for modern open-source data formats and tools.
