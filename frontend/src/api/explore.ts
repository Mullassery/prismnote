import axios from 'axios'

// A Data Explorer data source: a live kernel variable, a DuckDB-readable file
// (Parquet/CSV/JSON/Iceberg/Delta), or a raw DuckDB query.
export type Source =
  | { kind: 'var'; name: string }
  | { kind: 'file'; path: string }
  | { kind: 'sql'; query: string }

export type LogicalType = 'number' | 'string' | 'datetime' | 'bool' | 'array' | 'struct' | 'other'

export interface ColumnSchema {
  name: string
  dtype: string
  logical: LogicalType
  null_count: number
  null_pct: number
}

export interface SchemaResult {
  shape: [number, number]
  columns: ColumnSchema[]
  mem_bytes: number
}

export interface Overview {
  rows: number
  cols: number
  mem_bytes: number
  total_cells: number
  total_nulls: number
  null_pct: number
  duplicate_rows: number | null
  type_breakdown: Record<string, number>
  worst_columns: { name: string; null_pct: number }[]
  complete_columns: number
  constant_columns: number
  index_name: string | null
}

// One row per column — a richer df.describe(include='all').
export interface ColumnStat {
  name: string
  dtype: string
  logical: LogicalType
  count: number
  nulls: number
  null_pct: number
  distinct: number
  // numeric
  mean?: number
  std?: number
  min?: number
  q1?: number
  median?: number
  q3?: number
  max?: number
  sum?: number
  skew?: number | null
  kurtosis?: number | null
  // categorical
  top?: any
  freq?: number
}

export interface DescribeResult {
  columns: ColumnStat[]
  n: number
}

export interface Lineage {
  kind: 'variable' | 'file' | 'sql'
  shape?: [number, number]
  columns?: string[]
  // variable
  name?: string
  obj_type?: string | null
  // file
  path?: string
  format?: string
  size_bytes?: number
  modified?: string
  exists?: boolean
  // sql
  engine?: string
  query?: string
  references?: { type: string; target: string }[]
}

export type SortDir = 'asc' | 'desc'
export interface Sort {
  col: string
  dir: SortDir
}

export type FilterOp =
  | '==' | '!=' | '<' | '<=' | '>' | '>='
  | 'contains' | 'in' | 'isnull' | 'notnull'
export interface Filter {
  col: string
  op: FilterOp
  value?: any
}

export interface PageResult {
  columns: string[]
  data: any[][]
  total: number
}

export type ColumnProfile =
  | {
      kind: 'number'
      null_pct: number
      min?: number
      max?: number
      mean?: number
      median?: number
      std?: number
      q?: number[]
      hist: { counts: number[]; edges: number[] }
    }
  | { kind: 'datetime'; null_pct: number; min: any; max: any; cardinality: number }
  | {
      kind: 'category'
      null_pct: number
      cardinality: number
      top: { value: any; count: number }[]
    }
  | {
      kind: 'nested'
      subtype: 'array' | 'struct'
      null_pct: number
      count: number
      min_len: number | null
      max_len: number | null
      avg_len: number | null
      fields: string[] | null
    }

export interface Measure {
  col: string
  agg: 'sum' | 'mean' | 'count' | 'min' | 'max'
}

/** Either `{ var }` or `{ source }` identifies the target frame. */
type Target = { var: string } | { source: Source }

const post = <T,>(url: string, body: any) => axios.post<T>(url, body).then((r) => r.data)

export const exploreOverview = (t: Target) =>
  post<Overview>('/api/explore/overview', t)

export const exploreSchema = (t: Target) =>
  post<SchemaResult>('/api/explore/schema', t)

export const exploreDescribe = (t: Target) =>
  post<DescribeResult>('/api/explore/describe', t)

export const exploreLineage = (t: Target) =>
  post<Lineage>('/api/explore/lineage', t)

export const explorePage = (
  t: Target,
  opts: { offset: number; limit: number; sort?: Sort[]; filters?: Filter[]; search?: string },
) => post<PageResult>('/api/explore/page', { ...t, ...opts })

export const exploreProfile = (t: Target, col: string) =>
  post<ColumnProfile>('/api/explore/profile', { ...t, col })

export const exploreAggregate = (
  t: Target,
  opts: { dims: string[]; measures: Measure[]; filters?: Filter[]; limit?: number },
) => post<PageResult>('/api/explore/aggregate', { ...t, ...opts })

export const exploreExportCode = (
  t: Target,
  opts: { sort?: Sort[]; filters?: Filter[] },
) => post<{ code: string }>('/api/explore/export-code', { ...t, ...opts }).then((r) => r.code)
