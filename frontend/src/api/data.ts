import axios from 'axios'

export interface DbConnection {
  id: string
  name: string
  db_type: string
  host?: string | null
  port?: number | null
  database: string
  username?: string | null
  password?: string | null
  url?: string | null
  created_at: string
}

export interface QueryResult {
  columns: (string | number)[]
  rows: any[][]
  row_count: number
}

export const listDatabases = () =>
  axios.get<{ databases: DbConnection[] }>('/api/databases').then((r) => r.data.databases)

export const createDatabase = (c: Partial<DbConnection>) =>
  axios
    .post<DbConnection>('/api/databases', {
      id: '',
      name: c.name,
      db_type: c.db_type,
      host: c.host ?? null,
      port: c.port ?? null,
      database: c.database,
      username: c.username ?? null,
      password: c.password ?? null,
      url: c.url ?? null,
      created_at: '',
    })
    .then((r) => r.data)

export const deleteDatabase = (id: string) => axios.delete(`/api/databases/${id}`)

export const queryDatabase = (id: string, query: string) =>
  axios
    .post<QueryResult>(`/api/databases/${id}/query`, { connection_id: id, query })
    .then((r) => r.data)

export const listWarehouses = () =>
  axios.get<{ connections: any[] }>('/api/cloud-warehouses').then((r) => r.data.connections)

export const queryWarehouse = (id: string, query: string) =>
  axios
    .post<QueryResult>(`/api/cloud-warehouses/${id}/query`, { query })
    .then((r) => r.data)
