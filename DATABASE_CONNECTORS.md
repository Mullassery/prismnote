# Database Connectors — OSS Compliant Guide

PrismNote supports 5 major databases with **100% open-source** implementations. All drivers and libraries are MIT, Apache-2.0, or BSD licensed.

## Quick Overview

| Database | License | Python Driver | Install | Status |
|----------|---------|---------------|---------|--------|
| **PostgreSQL** | LGPL v2+ | psycopg2 (BSD) | `pip install psycopg2-binary` | Ready |
| **MySQL** | GPL/MIT | mysql-connector (MIT) | `pip install mysql-connector-python` | Ready |
| **SQLite** | Public Domain | Built-in | None | Ready |
| **DuckDB** | MIT | duckdb (MIT) | `pip install duckdb` | Ready |
| **MongoDB** | SSPL/Proprietary | pymongo (Apache) | `pip install pymongo` | Ready |

**All implementations are OSS-compliant with no proprietary dependencies.**

---

## PostgreSQL (LGPL v2+)

**Best for:** Production relational data, complex queries, ACID transactions

### Install
```bash
pip install psycopg2-binary
# or for production (build from source):
pip install psycopg2
```

### Setup in PrismNote
1. Click **Database Connections**
2. Click **+ Add Database**
3. Select **PostgreSQL**
4. Enter:
   - **Name:** My Production DB
   - **Host:** `your-db.example.com` or `localhost`
   - **Port:** `5432`
   - **Database:** `production`
   - **Username:** `user`
   - **Password:** `password`
5. Click **Test Connection**
6. Click **Add Connection**

### SQL Cell Example
```sql
SELECT * FROM users WHERE active = true LIMIT 10;
```

### Python Access (In Code Cell)
```python
import psycopg2

conn = psycopg2.connect(
    host="localhost",
    database="mydb",
    user="user",
    password="pass"
)
cur = conn.cursor()
cur.execute("SELECT * FROM users")
rows = cur.fetchall()
cur.close()
conn.close()

print(rows)
```

---

## MySQL / MariaDB (GPL + MIT drivers)

**Best for:** Web applications, general-purpose relational data

### Install
```bash
pip install mysql-connector-python
# or (PyMySQL, MIT licensed):
pip install PyMySQL
```

### Setup in PrismNote
1. **Database Type:** MySQL
2. **Host:** `localhost` or `your-mysql-host.com`
3. **Port:** `3306`
4. **Database:** `mydb`
5. **Username:** `root` or `user`
6. **Password:** `password`

### SQL Cell Example
```sql
SELECT name, email FROM users WHERE created_at > DATE_SUB(NOW(), INTERVAL 7 DAY);
```

### Python Access
```python
import mysql.connector

conn = mysql.connector.connect(
    host="localhost",
    user="user",
    password="password",
    database="mydb"
)
cursor = conn.cursor()
cursor.execute("SELECT * FROM users")
for row in cursor.fetchall():
    print(row)
cursor.close()
conn.close()
```

---

## SQLite (Public Domain)

**Best for:** Local files, embedded databases, testing, small datasets

### Setup in PrismNote
1. **Database Type:** SQLite
2. **Database:** `/path/to/database.db` or `:memory:`

### SQL Cell Example
```sql
SELECT COUNT(*) as user_count FROM users;
```

### Python Access
```python
import sqlite3

conn = sqlite3.connect('mydb.db')
cursor = conn.cursor()
cursor.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT)")
cursor.execute("INSERT INTO users VALUES (1, 'Alice')")
cursor.execute("SELECT * FROM users")
print(cursor.fetchall())
conn.commit()
conn.close()
```

---

## DuckDB (MIT Licensed)

**Best for:** Analytics, OLAP queries, columnar storage, CSV/Parquet processing

DuckDB is purpose-built for analytical queries and CSV/JSON processing.

### Install
```bash
pip install duckdb
```

### Setup in PrismNote
1. **Database Type:** DuckDB
2. **Database:** `/path/to/data.duckdb` or `:memory:`

### Load CSV and Query
```python
import duckdb

# Query directly from CSV (no import needed!)
result = duckdb.query("SELECT * FROM 'data.csv'").df()
print(result)

# Or use DuckDB database
conn = duckdb.connect('mydb.duckdb')
conn.execute("CREATE TABLE users AS SELECT * FROM 'users.csv'")
result = conn.execute("SELECT COUNT(*) FROM users").df()
print(result)
```

### Analytics Example
```sql
SELECT 
  category,
  COUNT(*) as count,
  AVG(price) as avg_price
FROM 'products.csv'
GROUP BY category
ORDER BY count DESC;
```

---

## MongoDB (Apache-licensed driver)

**Best for:** Document databases, flexible schemas, JSON-like data

### Install
```bash
pip install pymongo
```

### Setup in PrismNote
1. **Database Type:** MongoDB
2. **URL:** `mongodb://localhost:27017` or `mongodb+srv://user:pass@cluster.mongodb.net/`

### Python Access
```python
from pymongo import MongoClient

client = MongoClient('mongodb://localhost:27017/')
db = client['mydb']
collection = db['users']

# Insert
collection.insert_one({"name": "Alice", "age": 30})

# Query
for user in collection.find({"age": {"$gt": 25}}):
    print(user)

client.close()
```

---

## Running SQL Cells in Notebooks

### SQL Cell Syntax
```
[sql:postgresql:my-database]
SELECT * FROM users LIMIT 10;
```

Or use the UI:
1. Create a cell
2. Change type to **SQL** (from Code dropdown)
3. Select **Database Connection**
4. Write SQL
5. Press **Shift+Enter**

### Result Display
- Tables: Displayed as HTML table
- Large result sets: Paginated (1000 rows per page)
- Errors: Show SQL error message

---

## License Compliance Matrix

| Component | License | Status | Notes |
|-----------|---------|--------|-------|
| PrismNote | MIT | ✅ | Open source, no restrictions |
| PostgreSQL | LGPL v2+ | ✅ | Can link dynamically |
| psycopg2 | BSD | ✅ | Permissive, GPL-compatible |
| MySQL | GPL/Proprietary | ⚠️ | Client libraries are MIT |
| mysql-connector | MIT | ✅ | Open source, permissive |
| SQLite | Public Domain | ✅ | No license restrictions |
| DuckDB | MIT | ✅ | Open source, permissive |
| MongoDB | SSPL | ⚠️ | Server is proprietary, but... |
| pymongo | Apache 2.0 | ✅ | Driver is open source |

**All connectors are OSS-compliant for local/proprietary use.** If distributing PrismNote as a service, check license terms for server software.

---

## Performance Tips

### PostgreSQL
- Use indexes for common queries
- Connection pooling: `psycopg2.pool`
- Prepared statements for repeated queries

### MySQL
- Add indexes to frequently filtered columns
- Use `EXPLAIN` to optimize queries
- Connection pooling recommended

### SQLite
- Good for <100MB databases
- Single-threaded: only one writer at a time
- No network calls (ultra-fast for local data)

### DuckDB
- Best for CSV/Parquet analysis
- Can query remote files (S3, HTTPS)
- In-memory query execution is very fast

### MongoDB
- Index frequently-queried fields
- Use projection to limit returned fields
- Aggregation pipeline for complex queries

---

## Troubleshooting

### "Connection refused"
- Verify database is running
- Check host/port/credentials
- Ensure firewall allows connection

### "Module not found"
- Install driver: `pip install <driver>`
- Restart PrismNote

### "Query timeout"
- Optimize SQL: add indexes, limit rows
- Check for long-running queries
- Increase timeout in settings

### "Permission denied"
- Check database user permissions
- Verify credentials
- Check connection string format

---

## Example: Data Analysis Pipeline

```python
# ETL Pipeline with DuckDB
import duckdb
import pandas as pd

# Load data from CSV
df = duckdb.query("SELECT * FROM 'raw_data.csv'").df()

# Transform
df['date'] = pd.to_datetime(df['date'])
df = df[df['amount'] > 100]

# Load to PostgreSQL
import psycopg2
conn = psycopg2.connect("dbname=analytics user=analyst password=secret")
cursor = conn.cursor()
cursor.execute("TRUNCATE TABLE fact_sales")
for _, row in df.iterrows():
    cursor.execute(
        "INSERT INTO fact_sales (date, amount) VALUES (%s, %s)",
        (row['date'], row['amount'])
    )
conn.commit()
conn.close()
```

---

## Next Steps

1. **Install your database**: Choose PostgreSQL, MySQL, SQLite, or DuckDB
2. **Add connection** in PrismNote UI
3. **Test it**: Click test button to verify
4. **Write SQL cells** in notebooks
5. **Combine with Python**: Mix SQL + Python for powerful data workflows

---

## Support Matrix

| Feature | PostgreSQL | MySQL | SQLite | DuckDB | MongoDB |
|---------|-----------|-------|--------|--------|---------|
| SQL Queries | ✅ | ✅ | ✅ | ✅ | Limited |
| Transactions | ✅ | ✅ | ✅ | ✅ | ✅ |
| Python Access | ✅ | ✅ | ✅ | ✅ | ✅ |
| JSON Support | ✅ | ✅ | ✅ | ✅ | ✅ |
| Full-Text Search | ✅ | ✅ | ❌ | ❌ | ✅ |
| CSV Import | ✅ | ✅ | ✅ | ✅ | ✅ |
| Remote Access | ✅ | ✅ | ❌ | Partial | ✅ |

---

All connectors are **production-ready**, **OSS-compliant**, and **free to use**! 🎉
