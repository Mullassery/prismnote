use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::process::Stdio;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use uuid::Uuid;

/// A long-lived Python interpreter that keeps ONE shared namespace across every
/// cell, so variables/imports/functions defined in one cell are visible in the
/// next (proper notebook semantics). Code is fed to the driver over stdin as a
/// JSON-encoded string (one line per cell); the driver replies with a single
/// `__PRISM_RESULT__{...}` line carrying captured stdout/stderr, the repr of the
/// last expression, and any traceback.
const DRIVER: &str = r#"
import sys, json, io, ast, base64, traceback, contextlib, signal

_ns = {"__name__": "__main__"}

# Ensure SIGINT raises KeyboardInterrupt even if we inherited SIG_IGN from a
# backgrounded parent (nohup/&/service) — this is what makes "stop cell" work.
try:
    signal.signal(signal.SIGINT, signal.default_int_handler)
except Exception:
    pass

# Pretty output + rich rendering by default. All optional — a missing library
# must never stop the kernel from starting.
def _bootstrap():
    try:
        import matplotlib
        matplotlib.use("Agg")   # headless: figures are captured, never displayed
    except Exception:
        pass
    try:
        import rich
        from rich import pretty
        pretty.install()
        _ns["rich"] = rich
        from rich.pretty import pprint as _pp
        _ns["pprint"] = _pp
    except Exception:
        from pprint import pprint as _pp
        _ns["pprint"] = _pp
    try:
        import pandas as pd
        pd.set_option("display.max_columns", 50)
        pd.set_option("display.width", 120)
        _ns["pd"] = pd
    except Exception:
        pass

    # Dynamic input widgets (Databricks dbutils.widgets-style). prism.input/slider/
    # select/checkbox render a control and return its current value; the value
    # persists across runs and is set by the UI before re-running the cell.
    class _Prism:
        def __init__(self):
            self._vals = {}
            self._widgets = []
        def _set(self, name, value):
            self._vals[name] = value
        def _w(self, spec, default):
            spec = dict(spec)
            val = self._vals.get(spec["name"], default)
            spec["value"] = val
            self._widgets.append(spec)
            return val
        def input(self, name, default=""):
            return self._w({"type": "text", "name": name}, default)
        def slider(self, name, min=0, max=100, default=None):
            return self._w({"type": "slider", "name": name, "min": min, "max": max},
                           default if default is not None else min)
        def select(self, name, options, default=None):
            opts = list(options)
            return self._w({"type": "select", "name": name, "options": opts},
                           default if default is not None else (opts[0] if opts else None))
        def checkbox(self, name, default=False):
            return self._w({"type": "checkbox", "name": name}, default)
    _ns["prism"] = _Prism()

_bootstrap()

def _mime_bundle(val):
    """Build a Jupyter-style MIME bundle for a value (text + optional HTML +
    structured table data so the UI can chart it)."""
    bundle = {"text/plain": repr(val)}
    fn = getattr(val, "_repr_html_", None)   # pandas DataFrame, etc.
    if callable(fn):
        try:
            html = fn()
            if html:
                bundle["text/html"] = html
        except Exception:
            pass
    # Structured payload for the chart switcher (DataFrames only, capped rows).
    try:
        import pandas as _pd
        if isinstance(val, _pd.DataFrame):
            bundle["application/vnd.prismnote.df+json"] = json.loads(
                val.head(500).to_json(orient="split", date_format="iso")
            )
    except Exception:
        pass
    return bundle

def _capture_figures(outputs):
    """Emit any open matplotlib figures as image/png display_data, then clear."""
    try:
        import matplotlib.pyplot as plt
    except Exception:
        return
    try:
        for num in plt.get_fignums():
            fig = plt.figure(num)
            data = {}
            buf = io.BytesIO()
            fig.savefig(buf, format="png", bbox_inches="tight", dpi=110)
            buf.seek(0)
            data["image/png"] = base64.b64encode(buf.read()).decode("ascii")
            # Also emit a vector copy so the Visualization Pane can offer crisp
            # zoom and SVG export (an edge over raster-only plot panes).
            try:
                sbuf = io.StringIO()
                fig.savefig(sbuf, format="svg", bbox_inches="tight")
                data["image/svg+xml"] = sbuf.getvalue()
            except Exception:
                pass
            outputs.append({
                "output_type": "display_data",
                "data": data,
                "metadata": {},
            })
        plt.close("all")
    except Exception:
        pass

# Tee stdout: buffer it (for the final result) AND emit each chunk immediately as
# a __PRISM_STREAM__ line so the UI can show output live.
class _Tee:
    def __init__(self, buf):
        self.buf = buf
    def write(self, s):
        self.buf.write(s)
        if s:
            try:
                sys.__stdout__.write("__PRISM_STREAM__" + json.dumps(s) + "\n")
                sys.__stdout__.flush()
            except Exception:
                pass
        return len(s)
    def flush(self):
        pass

def _run(src):
    outputs = []
    out, err = io.StringIO(), io.StringIO()
    pr = _ns.get("prism")
    if pr is not None:
        pr._widgets = []  # collect widgets declared during this run
    try:
        tree = ast.parse(src, mode="exec")
        last = None
        if tree.body and isinstance(tree.body[-1], ast.Expr):
            last = ast.Expression(tree.body.pop().value)
        val = None
        with contextlib.redirect_stdout(_Tee(out)), contextlib.redirect_stderr(err):
            if tree.body:
                exec(compile(tree, "<cell>", "exec"), _ns)
            if last is not None:
                val = eval(compile(last, "<cell>", "eval"), _ns)
        so, se = out.getvalue(), err.getvalue()
        if pr is not None and getattr(pr, "_widgets", None):
            for w in pr._widgets:
                outputs.append({"output_type": "display_data",
                                "data": {"application/vnd.prismnote.widget+json": w}, "metadata": {}})
        if so:
            outputs.append({"output_type": "stream", "name": "stdout", "text": [so]})
        if se:
            outputs.append({"output_type": "stream", "name": "stderr", "text": [se]})
        _capture_figures(outputs)
        if last is not None and val is not None:
            outputs.append({"output_type": "execute_result", "data": _mime_bundle(val), "metadata": {}})
    except SystemExit:
        so = out.getvalue()
        if so:
            outputs.append({"output_type": "stream", "name": "stdout", "text": [so]})
    except BaseException:
        so = out.getvalue()
        if so:
            outputs.append({"output_type": "stream", "name": "stdout", "text": [so]})
        tb = traceback.format_exc()
        outputs.append({
            "output_type": "error",
            "ename": type(sys.exc_info()[1]).__name__,
            "evalue": str(sys.exc_info()[1]),
            "traceback": tb.splitlines(),
            "text": [tb],
        })
    return outputs

import types as _types
_HIDDEN = {"prism", "pd", "rich", "pprint"}

def _inspect():
    """Snapshot user-defined variables for the variable explorer."""
    out = []
    for name, v in list(_ns.items()):
        if name.startswith("_") or name in _HIDDEN:
            continue
        if isinstance(v, (_types.ModuleType, _types.FunctionType, _types.BuiltinFunctionType, type)):
            continue
        info = {"name": name, "type": type(v).__name__}
        try:
            if type(v).__name__ == "DataFrame":
                info["shape"] = list(v.shape)
                info["preview"] = f"DataFrame {v.shape[0]}x{v.shape[1]}: {list(v.columns)[:8]}"
            elif type(v).__name__ in ("ndarray",):
                info["shape"] = list(v.shape)
                info["preview"] = repr(v)[:120]
            elif hasattr(v, "__len__"):
                info["len"] = len(v)
                info["preview"] = repr(v)[:120]
            else:
                info["preview"] = repr(v)[:120]
        except Exception:
            info["preview"] = "<unrepresentable>"
        out.append(info)
    return out

def _as_dataframe(v):
    """Coerce a user value to a pandas DataFrame for exploration, or None."""
    import pandas as _pd
    if isinstance(v, _pd.DataFrame):
        return v
    # polars (best-effort)
    if type(v).__module__.startswith("polars") and hasattr(v, "to_pandas"):
        try:
            return v.to_pandas()
        except Exception:
            return None
    # numpy ndarray → DataFrame
    if type(v).__name__ == "ndarray":
        try:
            import numpy as _np
            a = _np.asarray(v)
            if a.ndim == 1:
                return _pd.DataFrame({"value": a})
            if a.ndim == 2:
                return _pd.DataFrame(a, columns=[f"c{i}" for i in range(a.shape[1])])
        except Exception:
            return None
    if isinstance(v, _pd.Series):
        return v.to_frame()
    return None


def _logical(s):
    """Logical type for a column. Accepts a Series (preferred, so we can sniff
    nested struct/array values) or a bare dtype."""
    import pandas as _pd
    dtype = s.dtype if hasattr(s, "dtype") else s
    if _pd.api.types.is_bool_dtype(dtype):
        return "bool"
    if _pd.api.types.is_numeric_dtype(dtype):
        return "number"
    if _pd.api.types.is_datetime64_any_dtype(dtype):
        return "datetime"
    # pyarrow-backed nested dtypes (DuckDB / pandas ArrowDtype)
    try:
        import pyarrow as _pa
        pat = getattr(dtype, "pyarrow_dtype", None)
        if pat is not None:
            if _pa.types.is_list(pat) or _pa.types.is_large_list(pat) or _pa.types.is_fixed_size_list(pat):
                return "array"
            if _pa.types.is_struct(pat) or _pa.types.is_map(pat):
                return "struct"
    except Exception:
        pass
    # object columns: sniff a non-null sample for Python list/dict (struct/array)
    if dtype == object and hasattr(s, "dropna"):
        for v in s.dropna().head(20):
            if isinstance(v, (list, tuple)):
                return "array"
            if isinstance(v, dict):
                return "struct"
            break
    return "string"


def _jsonable(x):
    """Make a scalar JSON-safe (handle NaN/NaT/numpy types)."""
    import pandas as _pd
    try:
        if x is None or (_pd.isna(x) if _np_isscalar(x) else False):
            return None
    except Exception:
        pass
    if hasattr(x, "item"):
        try:
            return x.item()
        except Exception:
            pass
    if isinstance(x, (int, float, str, bool)):
        return x
    return str(x)


def _np_isscalar(x):
    try:
        import numpy as _np
        return _np.isscalar(x) or x is None
    except Exception:
        return not hasattr(x, "__len__")


def _apply_filters(df, filters):
    import pandas as _pd
    for f in filters or []:
        col, op, val = f.get("col"), f.get("op"), f.get("value")
        if col not in df.columns:
            continue
        s = df[col]
        try:
            if op == "==":
                df = df[s == val]
            elif op == "!=":
                df = df[s != val]
            elif op == "<":
                df = df[s < val]
            elif op == "<=":
                df = df[s <= val]
            elif op == ">":
                df = df[s > val]
            elif op == ">=":
                df = df[s >= val]
            elif op == "contains":
                df = df[s.astype(str).str.contains(str(val), case=False, na=False)]
            elif op == "in":
                df = df[s.isin(val if isinstance(val, list) else [val])]
            elif op == "isnull":
                df = df[s.isna()]
            elif op == "notnull":
                df = df[s.notna()]
        except Exception:
            pass
    return df


def _duckdb_to_df(sql):
    """Run a DuckDB query and return a pandas DataFrame. DuckDB natively reads
    Parquet/CSV/JSON and, with extensions, Iceberg/Delta — so the Data Explorer
    works on open table/file formats, not just in-memory frames."""
    import duckdb as _ddb
    con = _ns.get("__prism_duck__")
    if con is None:
        con = _ddb.connect(database=":memory:")
        # best-effort: enable open table formats if the extensions are present
        for ext in ("httpfs", "iceberg", "delta"):
            try:
                con.execute(f"INSTALL {ext}; LOAD {ext};")
            except Exception:
                pass
        _ns["__prism_duck__"] = con
    return con.execute(sql).fetch_df()


def _read_file_sql(path):
    """Build a DuckDB scan expression for a file path based on its extension."""
    p = path.lower()
    q = path.replace("'", "''")
    if p.endswith(".parquet") or p.endswith(".pq"):
        return f"SELECT * FROM read_parquet('{q}')"
    if p.endswith(".csv") or p.endswith(".tsv") or p.endswith(".txt"):
        return f"SELECT * FROM read_csv_auto('{q}')"
    if p.endswith(".json") or p.endswith(".ndjson") or p.endswith(".jsonl"):
        return f"SELECT * FROM read_json_auto('{q}')"
    if p.endswith(".arrow") or p.endswith(".feather"):
        return f"SELECT * FROM read_parquet('{q}')"  # fallback; arrow often readable
    # An Iceberg table directory (has metadata/) — use iceberg_scan.
    return f"SELECT * FROM iceberg_scan('{q}')"


def _load_source(req):
    """Resolve a Data Explorer request to a pandas DataFrame from one of:
    - a live kernel variable  {"var": "df"}  or  {"source": {"kind":"var","name":"df"}}
    - a file (DuckDB)         {"source": {"kind":"file","path":"data.parquet"}}
    - a DuckDB SQL query      {"source": {"kind":"sql","query":"SELECT ..."}}
    Results for file/sql sources are cached in the kernel so paging is cheap."""
    src = req.get("source")
    if not src:
        name = req.get("var")
        v = _ns.get(name)
        if v is None:
            return None, f"variable '{name}' not found"
        df = _as_dataframe(v)
        return (df, None) if df is not None else (None, f"'{name}' is not tabular (got {type(v).__name__})")

    kind = src.get("kind")
    if kind == "var":
        v = _ns.get(src.get("name"))
        if v is None:
            return None, f"variable '{src.get('name')}' not found"
        df = _as_dataframe(v)
        return (df, None) if df is not None else (None, "not a tabular value")

    cache = _ns.setdefault("__prism_explore_cache__", {})
    key = json.dumps(src, sort_keys=True)
    if key in cache:
        return cache[key], None
    try:
        if kind == "file":
            df = _duckdb_to_df(_read_file_sql(src["path"]))
        elif kind == "sql":
            df = _duckdb_to_df(src["query"])
        else:
            return None, f"unknown source kind '{kind}'"
    except Exception as e:
        return None, str(e)
    cache[key] = df
    return df, None


def _sql_refs(query):
    """Best-effort extraction of upstream sources referenced by a DuckDB query:
    file readers (read_parquet/read_csv_auto/read_json_auto/iceberg_scan/
    delta_scan) and plain FROM/JOIN table names."""
    import re
    refs = []
    seen = set()
    for fn in ("read_parquet", "read_csv_auto", "read_csv", "read_json_auto",
               "read_json", "iceberg_scan", "delta_scan", "parquet_scan"):
        for m in re.finditer(fn + r"\(\s*'([^']+)'", query, re.IGNORECASE):
            t = m.group(1)
            if t not in seen:
                seen.add(t)
                refs.append({"type": fn, "target": t})
    for m in re.finditer(r"\b(?:from|join)\s+([A-Za-z_][\w.]*)", query, re.IGNORECASE):
        t = m.group(1)
        if t.lower() in ("read_parquet", "read_csv_auto", "read_csv", "read_json_auto",
                         "read_json", "iceberg_scan", "delta_scan", "parquet_scan"):
            continue
        if t not in seen:
            seen.add(t)
            refs.append({"type": "table", "target": t})
    return refs


def _file_format(path):
    p = path.lower()
    for ext, fmt in ((".parquet", "Parquet"), (".pq", "Parquet"), (".csv", "CSV"),
                     (".tsv", "TSV"), (".json", "JSON"), (".ndjson", "JSON"),
                     (".jsonl", "JSON"), (".arrow", "Arrow"), (".feather", "Arrow")):
        if p.endswith(ext):
            return fmt
    return "Iceberg/other"


def _explore(req):
    """Backend for the Data Explorer / Visualization Pane. Operates on a live
    variable, a DuckDB-readable file, or a DuckDB query; never executes
    arbitrary user code."""
    op = req.get("op")
    if op == "lineage":
        src = req.get("source")
        info = {}
        if not src:
            name = req.get("var")
            v = _ns.get(name)
            info = {"kind": "variable", "name": name, "obj_type": type(v).__name__ if v is not None else None}
        elif src.get("kind") == "var":
            v = _ns.get(src.get("name"))
            info = {"kind": "variable", "name": src.get("name"), "obj_type": type(v).__name__ if v is not None else None}
        elif src.get("kind") == "file":
            import os, datetime
            p = src.get("path", "")
            info = {"kind": "file", "path": p, "format": _file_format(p)}
            try:
                st = os.stat(p)
                info["size_bytes"] = int(st.st_size)
                info["modified"] = datetime.datetime.fromtimestamp(st.st_mtime).isoformat(timespec="seconds")
                info["exists"] = True
            except Exception:
                info["exists"] = False
        elif src.get("kind") == "sql":
            q = src.get("query", "")
            info = {"kind": "sql", "engine": "DuckDB", "query": q, "references": _sql_refs(q)}
        # attach the resolved shape so the UI can show "produces N×M"
        d, e = _load_source(req)
        if d is not None:
            info["shape"] = [int(d.shape[0]), int(d.shape[1])]
            info["columns"] = [str(c) for c in d.columns]
        return info

    df, err = _load_source(req)
    if err:
        return {"error": err}
    if df is None:
        return {"error": "could not load data source"}

    if op == "overview":
        # Table-level metadata — richer than a typical warehouse "details" pane.
        n = max(1, len(df))
        nulls_per_col = df.isna().sum()
        total_cells = int(df.shape[0]) * int(df.shape[1])
        total_nulls = int(nulls_per_col.sum())
        try:
            dup_rows = int(df.duplicated().sum())
        except Exception:
            dup_rows = None
        type_breakdown = {}
        for c in df.columns:
            lg = _logical(df[c])
            type_breakdown[lg] = type_breakdown.get(lg, 0) + 1
        # most-incomplete columns (highest null %), handy for data-quality triage
        worst = sorted(
            ({"name": str(c), "null_pct": round(100.0 * int(nulls_per_col[c]) / n, 2)} for c in df.columns),
            key=lambda d: d["null_pct"], reverse=True,
        )[:5]
        # statistical insights: fully-populated vs constant (zero-variance) columns
        complete_cols = int(sum(1 for c in df.columns if int(nulls_per_col[c]) == 0))
        constant_cols = 0
        for c in df.columns:
            try:
                if int(df[c].nunique(dropna=False)) <= 1:
                    constant_cols += 1
            except TypeError:
                pass
        return {
            "rows": int(df.shape[0]),
            "cols": int(df.shape[1]),
            "mem_bytes": int(df.memory_usage(deep=True).sum()),
            "total_cells": total_cells,
            "total_nulls": total_nulls,
            "null_pct": round(100.0 * total_nulls / max(1, total_cells), 2),
            "duplicate_rows": dup_rows,
            "type_breakdown": type_breakdown,
            "worst_columns": worst,
            "complete_columns": complete_cols,
            "constant_columns": constant_cols,
            "index_name": str(df.index.name) if df.index.name is not None else None,
        }

    if op == "describe":
        # One-pass per-column statistics table (a richer df.describe(include='all')).
        n = max(1, len(df))
        out = []
        for c in df.columns:
            s = df[c]
            lg = _logical(s)
            try:
                nulls = int(s.isna().sum())
            except (ValueError, TypeError):
                nulls = int(sum(1 for v in s if v is None))
            rec = {
                "name": str(c), "dtype": str(s.dtype), "logical": lg,
                "count": int(len(s) - nulls), "nulls": nulls,
                "null_pct": round(100.0 * nulls / n, 2),
            }
            try:
                rec["distinct"] = int(s.nunique(dropna=True))
            except TypeError:
                rec["distinct"] = int(s.dropna().astype(str).nunique())
            if lg == "number":
                sv = s.dropna()
                if len(sv):
                    rec.update({
                        "mean": _jsonable(sv.mean()), "std": _jsonable(sv.std()),
                        "min": _jsonable(sv.min()), "q1": _jsonable(sv.quantile(0.25)),
                        "median": _jsonable(sv.median()), "q3": _jsonable(sv.quantile(0.75)),
                        "max": _jsonable(sv.max()), "sum": _jsonable(sv.sum()),
                        "skew": _jsonable(sv.skew()) if len(sv) > 2 else None,
                        "kurtosis": _jsonable(sv.kurtosis()) if len(sv) > 3 else None,
                    })
            elif lg not in ("array", "struct"):
                try:
                    vc = s.value_counts(dropna=True)
                    if len(vc):
                        rec["top"] = _jsonable(vc.index[0])
                        rec["freq"] = int(vc.iloc[0])
                except TypeError:
                    pass
            out.append(rec)
        return {"columns": out, "n": int(len(df))}

    if op == "schema":
        cols = []
        n = max(1, len(df))
        for c in df.columns:
            s = df[c]
            nulls = int(s.isna().sum())
            cols.append({
                "name": str(c),
                "dtype": str(s.dtype),
                "logical": _logical(s),
                "null_count": nulls,
                "null_pct": round(100.0 * nulls / n, 2),
            })
        return {
            "shape": [int(df.shape[0]), int(df.shape[1])],
            "columns": cols,
            "mem_bytes": int(df.memory_usage(deep=True).sum()),
        }

    if op == "page":
        offset = int(req.get("offset", 0))
        limit = min(int(req.get("limit", 100)), 500)
        sub = _apply_filters(df, req.get("filters"))
        search = req.get("search")
        if search:
            mask = sub.apply(lambda r: r.astype(str).str.contains(str(search), case=False, na=False).any(), axis=1)
            sub = sub[mask]
        sort = req.get("sort") or []
        if sort:
            by = [s["col"] for s in sort if s.get("col") in sub.columns]
            asc = [s.get("dir", "asc") != "desc" for s in sort if s.get("col") in sub.columns]
            if by:
                sub = sub.sort_values(by=by, ascending=asc, kind="mergesort")
        total = int(len(sub))
        window = sub.iloc[offset:offset + limit]
        payload = json.loads(window.to_json(orient="split", date_format="iso"))
        # drop the pandas index from the split payload; keep columns + data
        return {"columns": [str(c) for c in payload["columns"]], "data": payload["data"], "total": total}

    if op == "profile":
        col = req.get("col")
        if col not in df.columns:
            return {"error": f"column '{col}' not found"}
        s = df[col]
        n = max(1, len(s))
        lg = _logical(s)
        # nulls in nested columns can be array-like; count element-wise safely
        try:
            null_count = int(s.isna().sum())
        except (ValueError, TypeError):
            null_count = int(sum(1 for v in s if v is None))
        null_pct = round(100.0 * null_count / n, 2)
        if lg in ("array", "struct"):
            sv = [v for v in s if v is not None]
            lens = [len(v) for v in sv if hasattr(v, "__len__")]
            keys = {}
            if lg == "struct":
                for v in sv[:1000]:
                    if isinstance(v, dict):
                        for k in v.keys():
                            keys[str(k)] = keys.get(str(k), 0) + 1
            return {
                "kind": "nested", "subtype": lg, "null_pct": null_pct,
                "count": len(sv),
                "min_len": min(lens) if lens else None,
                "max_len": max(lens) if lens else None,
                "avg_len": round(sum(lens) / len(lens), 2) if lens else None,
                "fields": sorted(keys.keys())[:30] if keys else None,
            }
        if lg == "number":
            sv = s.dropna()
            if len(sv) == 0:
                return {"kind": "number", "null_pct": null_pct, "hist": {"counts": [], "edges": []}}
            import numpy as _np
            counts, edges = _np.histogram(sv.values, bins=min(20, max(1, len(sv.unique()))))
            q = sv.quantile([0.25, 0.5, 0.75]).tolist()
            return {
                "kind": "number", "null_pct": null_pct,
                "min": _jsonable(sv.min()), "max": _jsonable(sv.max()),
                "mean": _jsonable(sv.mean()), "median": _jsonable(sv.median()),
                "std": _jsonable(sv.std()), "q": [_jsonable(x) for x in q],
                "hist": {"counts": [int(c) for c in counts], "edges": [float(e) for e in edges]},
            }
        if lg == "datetime":
            sv = s.dropna()
            return {
                "kind": "datetime", "null_pct": null_pct,
                "min": _jsonable(sv.min()) if len(sv) else None,
                "max": _jsonable(sv.max()) if len(sv) else None,
                "cardinality": int(sv.nunique()),
            }
        try:
            vc = s.value_counts(dropna=True).head(20)
            card = int(s.nunique(dropna=True))
            top = [{"value": _jsonable(idx), "count": int(cnt)} for idx, cnt in vc.items()]
        except TypeError:
            # unhashable values that slipped past the sniff — stringify them
            ss = s.dropna().astype(str)
            vc = ss.value_counts().head(20)
            card = int(ss.nunique())
            top = [{"value": idx, "count": int(cnt)} for idx, cnt in vc.items()]
        return {
            "kind": "category", "null_pct": null_pct,
            "cardinality": card, "top": top,
        }

    if op == "aggregate":
        dims = [d for d in (req.get("dims") or []) if d in df.columns]
        measures = req.get("measures") or []
        sub = _apply_filters(df, req.get("filters"))
        limit = min(int(req.get("limit", 5000)), 50000)
        if not measures:
            # no measures: just count rows per dim combo
            if dims:
                g = sub.groupby(dims, dropna=False).size().reset_index(name="count")
            else:
                g = __import__("pandas").DataFrame({"count": [len(sub)]})
        else:
            aggmap = {}
            rename = {}
            for m in measures:
                c, a = m.get("col"), m.get("agg", "sum")
                if c in sub.columns or a == "count":
                    aggmap.setdefault(c, []).append(a)
            if dims:
                g = sub.groupby(dims, dropna=False).agg(aggmap)
                g.columns = ["_".join(map(str, t)).strip("_") for t in g.columns.to_flat_index()]
                g = g.reset_index()
            else:
                agg_series = sub.agg(aggmap)
                g = __import__("pandas").DataFrame(agg_series).T.reset_index(drop=True)
                g.columns = [str(c) for c in g.columns]
        g = g.head(limit)
        payload = json.loads(g.to_json(orient="split", date_format="iso"))
        return {"columns": [str(c) for c in payload["columns"]], "data": payload["data"], "total": int(len(g))}

    return {"error": f"unknown op '{op}'"}


def _main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            src = json.loads(line)
        except Exception:
            continue
        if src == "__PRISM_INSPECT__":
            sys.stdout.write("__PRISM_RESULT__" + json.dumps({"inspect": _inspect()}) + "\n")
            sys.stdout.flush()
            continue
        if isinstance(src, dict) and src.get("__prism_cmd__") == "explore":
            try:
                res = _explore(src.get("req", {}))
            except Exception as e:
                res = {"error": str(e)}
            sys.stdout.write("__PRISM_RESULT__" + json.dumps({"explore": res}) + "\n")
            sys.stdout.flush()
            continue
        outputs = _run(src)
        sys.stdout.write("__PRISM_RESULT__" + json.dumps({"outputs": outputs}) + "\n")
        sys.stdout.flush()

_main()
"#;

const RESULT_PREFIX: &str = "__PRISM_RESULT__";
const STREAM_PREFIX: &str = "__PRISM_STREAM__";

pub struct KernelManager {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
    kernel_id: String,
    execution_count: usize,
    timeout: Duration,
    /// Current interpreter PID, shared so the API can SIGINT a running cell
    /// without taking the kernel lock (which a running execute() holds).
    pid: Arc<AtomicI32>,
}

impl KernelManager {
    pub fn new() -> Result<Self> {
        // Verify python is available before we commit to a long-lived process.
        let check = std::process::Command::new("python")
            .arg("-c")
            .arg("print('ok')")
            .output();
        if check.is_err() || !String::from_utf8_lossy(&check?.stdout).contains("ok") {
            return Err(anyhow!("python not found on PATH"));
        }

        let (child, stdin, stdout) = Self::spawn_process()?;
        let pid = Arc::new(AtomicI32::new(child.id().map(|p| p as i32).unwrap_or(0)));
        Ok(KernelManager {
            child: Some(child),
            stdin: Some(stdin),
            stdout: Some(stdout),
            kernel_id: Uuid::new_v4().to_string(),
            execution_count: 0,
            timeout: Duration::from_secs(60),
            pid,
        })
    }

    /// Shareable handle to the live interpreter PID (0 when not running).
    pub fn pid_handle(&self) -> Arc<AtomicI32> {
        self.pid.clone()
    }

    fn spawn_process() -> Result<(Child, ChildStdin, BufReader<ChildStdout>)> {
        let mut child = Command::new("python")
            .arg("-u") // unbuffered, so we see the result line immediately
            .arg("-c")
            .arg(DRIVER)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow!("failed to start python kernel: {}", e))?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("no kernel stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("no kernel stdout"))?;
        Ok((child, stdin, BufReader::new(stdout)))
    }

    /// Restart the interpreter, wiping the shared namespace. Used after a timeout
    /// (the old process is left in an unknown state) and for explicit restarts.
    pub fn restart(&mut self) -> Result<()> {
        if let Some(mut c) = self.child.take() {
            let _ = c.start_kill();
        }
        let (child, stdin, stdout) = Self::spawn_process()?;
        self.pid.store(child.id().map(|p| p as i32).unwrap_or(0), Ordering::SeqCst);
        self.child = Some(child);
        self.stdin = Some(stdin);
        self.stdout = Some(stdout);
        self.execution_count = 0;
        Ok(())
    }

    pub async fn execute(&mut self, code: &str) -> Result<(Vec<String>, Vec<Value>)> {
        self.execute_streaming(code, None).await
    }

    /// Like `execute`, but forwards live stdout chunks to `stream` as they happen.
    pub async fn execute_streaming(
        &mut self,
        code: &str,
        stream: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    ) -> Result<(Vec<String>, Vec<Value>)> {
        self.execution_count += 1;

        // pip installs run as a one-off so we don't block the shared interpreter.
        if code.trim().starts_with("pip install") || code.trim().starts_with("!pip") {
            return self.handle_package_install(code).await;
        }

        match tokio::time::timeout(self.timeout, self.execute_internal(code, stream)).await {
            Ok(Ok(outputs)) => Ok((vec![], outputs)),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // The interpreter is mid-execution and the pipe is desynced; the
                // only safe recovery is a restart.
                let _ = self.restart();
                Err(anyhow!(
                    "Execution timed out after {:?} — kernel restarted, all variables cleared.",
                    self.timeout
                ))
            }
        }
    }

    async fn execute_internal(
        &mut self,
        code: &str,
        stream: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    ) -> Result<Vec<Value>> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("kernel not running"))?;

        // One JSON-encoded line per cell (newlines in code become \n inside the
        // JSON string, so the framing stays line-oriented).
        let msg = serde_json::to_string(code)?;
        stdin.write_all(msg.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        let reader = self
            .stdout
            .as_mut()
            .ok_or_else(|| anyhow!("kernel not running"))?;

        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                return Err(anyhow!("kernel exited unexpectedly"));
            }
            if let Some(rest) = line.strip_prefix(RESULT_PREFIX) {
                let res: Value = serde_json::from_str(rest.trim_end())?;
                return Self::build_outputs(res);
            }
            if let Some(rest) = line.strip_prefix(STREAM_PREFIX) {
                // live stdout chunk (JSON-encoded string) → forward if streaming
                if let Some(tx) = &stream {
                    if let Ok(Value::String(s)) = serde_json::from_str::<Value>(rest.trim_end()) {
                        let _ = tx.send(s);
                    }
                }
                continue;
            }
            // Anything else is stray driver chatter — ignore it.
        }
    }

    /// The driver already builds Jupyter-style output objects (stream,
    /// execute_result with a MIME bundle, display_data for figures, and error
    /// with a traceback). Just hand them through.
    fn build_outputs(res: Value) -> Result<Vec<Value>> {
        match res.get("outputs") {
            Some(Value::Array(a)) => Ok(a.clone()),
            _ => Ok(vec![]),
        }
    }

    async fn handle_package_install(&self, code: &str) -> Result<(Vec<String>, Vec<Value>)> {
        let clean_code = code.replace("!pip install", "pip install").replace("!pip", "pip");

        let output = Command::new("python")
            .arg("-m")
            .arg("pip")
            .arg("install")
            .args(clean_code.split_whitespace().skip(2))
            .output()
            .await
            .map_err(|e| anyhow!("Package installation failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut outputs = vec![];
        if !stdout.is_empty() {
            outputs.push(json!({
                "output_type": "stream",
                "name": "stdout",
                "text": format!("✅ Package installed\n{}", stdout)
            }));
        }
        if !stderr.is_empty() {
            outputs.push(json!({"output_type": "stream", "name": "stderr", "text": stderr}));
        }
        Ok((vec![stdout], outputs))
    }

    /// Snapshot user variables (for the variable explorer). Does not run user
    /// code or bump the execution counter.
    pub async fn inspect(&mut self) -> Result<Value> {
        let stdin = self.stdin.as_mut().ok_or_else(|| anyhow!("kernel not running"))?;
        let msg = serde_json::to_string("__PRISM_INSPECT__")?;
        stdin.write_all(msg.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        let reader = self.stdout.as_mut().ok_or_else(|| anyhow!("kernel not running"))?;
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                return Err(anyhow!("kernel exited unexpectedly"));
            }
            if let Some(rest) = line.strip_prefix(RESULT_PREFIX) {
                let res: Value = serde_json::from_str(rest.trim_end())?;
                return Ok(res.get("inspect").cloned().unwrap_or(Value::Array(vec![])));
            }
            // ignore stray stream frames
        }
    }

    /// Run a Data Explorer query against the live namespace (schema/page/profile/
    /// aggregate). Like `inspect()`, it does not run user code or bump the counter.
    pub async fn explore(&mut self, req: Value) -> Result<Value> {
        let stdin = self.stdin.as_mut().ok_or_else(|| anyhow!("kernel not running"))?;
        let cmd = json!({ "__prism_cmd__": "explore", "req": req });
        let msg = serde_json::to_string(&cmd)?;
        stdin.write_all(msg.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        let reader = self.stdout.as_mut().ok_or_else(|| anyhow!("kernel not running"))?;
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                return Err(anyhow!("kernel exited unexpectedly"));
            }
            if let Some(rest) = line.strip_prefix(RESULT_PREFIX) {
                let res: Value = serde_json::from_str(rest.trim_end())?;
                return Ok(res.get("explore").cloned().unwrap_or(json!({ "error": "no result" })));
            }
            // ignore stray stream frames
        }
    }

    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout = duration;
    }

    pub fn execution_count(&self) -> usize {
        self.execution_count
    }

    pub fn kernel_id(&self) -> &str {
        &self.kernel_id
    }
}

impl Drop for KernelManager {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
        }
    }
}
