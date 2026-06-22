# Data Connectors & OSS Licensing

PrismNote's core is **MIT** and **vendors no database/warehouse drivers**. Real SQL
execution runs **inside the notebook's Python kernel** using connector libraries you
install yourself — so you choose what's on your machine, and the project stays
permissively licensed.

When a connector isn't installed, the query returns a clear error telling you exactly
what to `pip install`.

## Databases (`%sql` targets / Database connections)

| Source | Library | License |
|---|---|---|
| SQLite | `sqlite3` (stdlib) | Python (PSF) |
| DuckDB | `duckdb` | MIT |
| PostgreSQL | `pg8000` (pure-Python) | BSD-3-Clause |
| MySQL / MariaDB | `PyMySQL` | MIT |

```bash
pip install duckdb pg8000 pymysql        # as needed
```

In-cell, `%sql` uses **DuckDB**, which can also query any pandas DataFrame defined in
another cell by name:

```python
sales = pd.read_csv("sales.csv")
```
```sql
%sql
SELECT region, SUM(amount) AS total FROM sales GROUP BY region
```

## Cloud warehouses

All use the vendor's **official open-source** client (installed on demand):

| Warehouse | Library | License |
|---|---|---|
| Snowflake | `snowflake-connector-python[pandas]` | Apache-2.0 |
| BigQuery | `google-cloud-bigquery` (+ `db-dtypes`) | Apache-2.0 |
| Redshift | `redshift_connector` | Apache-2.0 |
| Databricks SQL | `databricks-sql-connector` | Apache-2.0 |
| Athena | `pyathena` | MIT |
| Trino | `trino` | Apache-2.0 |
| Presto | `presto-python-client` | Apache-2.0 |
| Azure Synapse | `pyodbc` | MIT ⚠️ |

⚠️ **Azure Synapse** uses `pyodbc` (MIT), but it needs Microsoft's **ODBC Driver for
SQL Server**, which is **proprietary** (free to use, not OSS). It's the only connector
that depends on a non-open driver — all others are fully open-source.

```bash
# examples
pip install "snowflake-connector-python[pandas]"
pip install google-cloud-bigquery db-dtypes
pip install databricks-sql-connector
```

## Why through the kernel?

- **No vendored drivers** → the MIT core has no heavyweight/native or
  ambiguously-licensed dependencies.
- **You control the environment** — install only what you use; upgrade independently.
- Results come back as pandas DataFrames, so they render as tables **and** feed the
  Table/Bar/Line chart switcher.
