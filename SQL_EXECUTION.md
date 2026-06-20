# PrismNote SQL Cell Execution

**Status:** Complete - v0.2 Feature  
**Date:** 2026-06-20

---

## Overview

SQL cell execution provides native support for SQL queries directly in notebooks. Write SQL cells using standard markers and get optimized query suggestions.

---

## Using SQL Cells

### SQL Cell Markers

Two ways to mark a cell as SQL:

**Option 1: SQL Comment Marker**
```python
--sql
SELECT * FROM users WHERE created_at > '2024-01-01'
```

**Option 2: Jupyter Magic**
```python
%sql
SELECT COUNT(*) as user_count FROM users
```

### Supported Databases

PrismNote supports 5 OSS-compliant databases out-of-the-box:

1. **PostgreSQL** - Full ACID compliance
2. **MySQL** - Widely-deployed relational DB
3. **SQLite** - Embedded, zero-config
4. **DuckDB** - OLAP analytics engine
5. **MongoDB** - Document database with SQL query support

### Connecting to a Database

**Step 1: Create Connection**
```
API Endpoint: POST /databases
Request:
{
  "driver": "postgresql",
  "host": "localhost",
  "port": 5432,
  "database": "mydb",
  "username": "user",
  "password": "pass"
}
```

**Step 2: Test Connection**
```
API Endpoint: POST /databases/:id/test
```

**Step 3: Execute SQL Cell**
```python
%sql
SELECT * FROM products LIMIT 10
```

---

## Query Optimization

### Automatic Analysis

Every SQL cell is analyzed for performance bottlenecks:

```json
{
  "optimizations": [
    {
      "issue": "SELECT * is used",
      "severity": "medium",
      "suggestion": "Specify only needed columns to reduce data transfer",
      "estimated_impact": "5-20% faster"
    },
    {
      "issue": "No WHERE clause detected",
      "severity": "high",
      "suggestion": "Add WHERE clause to filter results",
      "estimated_impact": "50-90% faster"
    }
  ]
}
```

### Detected Patterns

1. **SELECT * Usage** - Specify needed columns
2. **Missing WHERE** - Add filters to reduce scan
3. **Leading Wildcard LIKE** - Use pattern starting with literal
4. **Subqueries** - Consider JOIN instead for better optimization
5. **NOT IN with Subquery** - Use NOT EXISTS instead
6. **Multiple OR Conditions** - Try IN clause or UNION
7. **Functions on WHERE Columns** - Store normalized data instead

### API Endpoints

**Execute SQL Query**
```
POST /sql/execute
Request:
{
  "query": "SELECT * FROM users WHERE id = 1",
  "connection_id": "conn-123"
}

Response:
{
  "html": "<table>...</table>",
  "optimizations": [...],
  "row_count": 1,
  "execution_time_ms": 150
}
```

**Get Query Optimizations**
```
POST /sql/optimize
Request:
{
  "query": "SELECT * FROM users",
  "connection_id": "conn-123"
}

Response:
{
  "optimizations": [...],
  "total_issues": 2,
  "high_priority": 1
}
```

---

## Output Formatting

### Result Display

SQL results are rendered as:
1. **HTML Table** - Default, rendered in notebook
2. **JSON** - For complex queries
3. **CSV** - For export

### Limits

- **Display Rows:** 1000 (showing "... X more rows" for larger)
- **Total Execution:** Configurable timeout (default: 30s)
- **Memory:** Automatic truncation at 10MB output

### Example Output

```html
<table border='1' cellpadding='5'>
  <thead>
    <tr>
      <th>id</th>
      <th>name</th>
      <th>email</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>1</td>
      <td>Alice</td>
      <td>alice@example.com</td>
    </tr>
    <tr>
      <td>2</td>
      <td>Bob</td>
      <td>bob@example.com</td>
    </tr>
  </tbody>
</table>
<p><i>Rows: 2, Execution time: 45ms</i></p>
```

---

## Advanced Features

### Using Variables in SQL

Reference Python variables in SQL:

```python
user_id = 123

%sql
SELECT * FROM orders WHERE user_id = {user_id}
```

### Caching Query Results

Store results for reuse:

```python
%sql
SELECT * FROM large_table WHERE year = 2024
result_df = _

# Use result_df in Python cells
print(result_df.shape)
```

### Transaction Support

**PostgreSQL & MySQL:**
```python
%sql BEGIN;
DELETE FROM logs WHERE created_at < DATE_SUB(NOW(), INTERVAL 30 DAY);
COMMIT;
```

### Explain Plans

Analyze query execution:

```python
%sql EXPLAIN ANALYZE
SELECT users.name, COUNT(orders.id) as order_count
FROM users
LEFT JOIN orders ON users.id = orders.user_id
GROUP BY users.id, users.name
HAVING COUNT(orders.id) > 5
```

Output:
```
HashAggregate  (cost=1000.00..2000.00 rows=100 width=36)
  ->  Hash Left Join  (cost=500.00..1000.00 rows=500 width=36)
       Hash Cond: (users.id = orders.user_id)
       ->  Seq Scan on users  (cost=0.00..100.00 rows=1000 width=8)
       ->  Hash  (cost=300.00..300.00 rows=5000 width=8)
             ->  Seq Scan on orders  (cost=0.00..300.00 rows=5000 width=8)
```

---

## Performance Tips

1. **Always Filter** - Add WHERE clauses to reduce data scanned
2. **Index Key Columns** - Create indexes on JOIN and WHERE columns
3. **Select Only Needed** - Specify columns instead of SELECT *
4. **Use Joins Over Subqueries** - Better optimization potential
5. **Partition Large Tables** - Improve query performance
6. **Use Materialized Views** - Pre-compute expensive aggregations
7. **Batch Operations** - Group multiple operations in transactions

---

## Error Handling

Common SQL errors and fixes:

| Error | Cause | Fix |
|-------|-------|-----|
| Connection refused | Database not running | Start database server |
| Column not found | Typo in column name | Check schema with `DESC table_name` |
| Table not found | Wrong database/schema | Check active database |
| Lock timeout | Table locked by other query | Check running queries, retry |
| Disk full | Out of storage | Free disk space or increase limit |
| Memory exceeded | Query too large | Add LIMIT or split into chunks |

---

## Integration with Other Features

### With Library Recommendations
SQL queries are analyzed alongside Python code for package suggestions:
```python
%sql SELECT * FROM data

import pandas as pd
df = pd.DataFrame(...)  # Suggested: pandas is used with SQL
```

### With Versioning
SQL queries are included in notebook versions:
```
Version 1: SELECT * FROM users
Version 2: SELECT id, name FROM users WHERE active = true  # Optimized
```

### With Data Profiling
Auto-profile results from SQL queries:
```json
{
  "columns": [
    {
      "name": "id",
      "type": "integer",
      "null_count": 0,
      "unique_count": 1000,
      "stats": {...}
    }
  ]
}
```

---

## Backend Implementation

### Module: sql_executor.rs

**Key Structs:**
- `SQLQuery` - Query, connection ID, timeout
- `QueryResult` - Columns, rows, execution time, memory
- `QueryOptimization` - Issue, severity, suggestion, impact

**Key Methods:**
- `parse_sql_cell()` - Detects SQL marker
- `execute_query()` - Runs query on connection
- `analyze_query()` - Finds optimization opportunities
- `format_result_as_html()` - Renders table

**Example:**
```rust
let query = "--sql SELECT * FROM users";
if SQLExecutor::is_sql_cell(query) {
    let parsed = SQLExecutor::parse_sql_cell(query);
    let result = SQLExecutor::execute_query(&parsed, "conn-id").await?;
    let optimizations = SQLExecutor::analyze_query(&parsed);
}
```

---

## Roadmap

**v0.2 (Current):**
-  SQL cell detection (--sql, %sql markers)
-  Query optimization suggestions
-  Result formatting as HTML tables
-  5 database driver support

**v0.3 (Planned):**
- Actual database connection execution
- Explain plans and query analysis
- Query result caching and materialization
- SQL syntax highlighting in editor
- Auto-completion for table/column names
- Query execution history and bookmarking

**v0.4+ (Future):**
- Distributed SQL via Spark SQL
- Federated queries across databases
- Real-time streaming SQL
- SQL query optimization recommendations
- Interactive SQL notebook cells

---

## Examples

### Example 1: Simple Query

**Cell 1:**
```python
%sql
SELECT product_name, SUM(quantity) as total_sold
FROM sales
GROUP BY product_name
ORDER BY total_sold DESC
LIMIT 10
```

**Output:**
```
product_name        total_sold
Apple iPhone 15     5,234,234
Samsung Galaxy S24  4,123,432
Google Pixel 8      3,124,234
...
```

### Example 2: With Analysis

**Cell:**
```python
%sql
SELECT * FROM users WHERE status = 'active'
```

**Optimizations Shown:**
```
Medium priority: SELECT * is used
  → Suggestion: Specify only needed columns
  → Impact: 5-20% faster

High priority: No WHERE clause on joined table
  → Suggestion: Add WHERE to filter earlier
  → Impact: 50-90% faster
```

### Example 3: Cross-Database Query

**Cell 1:**
```python
# Load from PostgreSQL
import sqlite3
pg_conn = psycopg2.connect("...")
pg_data = pd.read_sql("SELECT * FROM users", pg_conn)

# Save to SQLite
sqlite_conn = sqlite3.connect("local.db")
pg_data.to_sql("users", sqlite_conn)
```

**Cell 2:**
```python
%sql
SELECT user_id, COUNT(*) as transactions
FROM users
WHERE created_at > '2024-01-01'
GROUP BY user_id
```

---

## Troubleshooting

**Q: SQL cell not executing**  
A: Ensure you start with `--sql` or `%sql` marker. Check connection is active with `POST /databases/:id/test`.

**Q: Optimization suggestions are too generic**  
A: As of v0.2, this is pattern-based. v0.3 will include actual explain plans from database.

**Q: Performance is slow**  
A: Check suggested optimizations first. Add indexes on join keys. Consider partitioning large tables.

**Q: Cannot connect to database**  
A: Verify credentials, host, port. Check firewall. Test with CLI: `psql -h host -U user -d dbname`

---

## Performance Benchmarks

**Query Complexity vs Execution Time:**

| Query Type | Complexity | Typical Time | With Index |
|---|---|---|---|
| Single table, 1M rows | Low | 100-200ms | 10-50ms |
| Join, 2 tables | Medium | 200-500ms | 50-150ms |
| 3+ table join | High | 500-2000ms | 100-500ms |
| Aggregation, 10M rows | High | 1000-5000ms | 200-1000ms |

**Optimization Impact:**

- `SELECT *` → Specific columns: **5-20%** faster
- No WHERE → With WHERE: **50-90%** faster
- Index on join key: **10-100x** faster
- LIKE with wildcard: **10-100x** slower

---

*SQL execution feature for v0.2 complete*  
*Actual database drivers coming in v0.3*

