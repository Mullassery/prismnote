import React, { useState, useEffect } from 'react'
import { Database, Play, Download, Upload, Settings } from 'lucide-react'
import { Button } from './common/Button'
import { Card, CardBody, CardHeader } from './common/Card'

interface Table {
  name: string
  rowCount: number
  sizeBytes: number
}

interface QueryResult {
  columns: string[]
  rows: Array<any[]>
  rowCount: number
  executionTimeMs: number
}

export default function DuckDBExplorer() {
  const [tables, setTables] = useState<Table[]>([])
  const [query, setQuery] = useState('')
  const [result, setResult] = useState<QueryResult | null>(null)
  const [isExecuting, setIsExecuting] = useState(false)
  const [selectedTable, setSelectedTable] = useState<string | null>(null)
  const [extensions, setExtensions] = useState<string[]>([])

  useEffect(() => {
    loadTables()
    loadExtensions()
  }, [])

  const loadTables = async () => {
    try {
      const response = await fetch('/api/duckdb/tables')
      if (response.ok) {
        const data = await response.json()
        setTables(data)
      }
    } catch (error) {
      console.error('Failed to load tables:', error)
    }
  }

  const loadExtensions = async () => {
    try {
      const response = await fetch('/api/duckdb/extensions')
      if (response.ok) {
        const data = await response.json()
        setExtensions(data)
      }
    } catch (error) {
      console.error('Failed to load extensions:', error)
    }
  }

  const executeQuery = async () => {
    if (!query.trim()) return

    setIsExecuting(true)
    try {
      const response = await fetch('/api/duckdb/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query }),
      })

      if (response.ok) {
        const data = await response.json()
        setResult(data)
      }
    } catch (error) {
      console.error('Query execution failed:', error)
    }
    setIsExecuting(false)
  }

  const queryTable = async (tableName: string) => {
    setQuery(`SELECT * FROM ${tableName} LIMIT 100`)
    setSelectedTable(tableName)
  }

  const loadParquet = async (file: File) => {
    const formData = new FormData()
    formData.append('file', file)
    formData.append('table_name', file.name.replace('.parquet', ''))

    try {
      const response = await fetch('/api/duckdb/load-parquet', {
        method: 'POST',
        body: formData,
      })

      if (response.ok) {
        await loadTables()
      }
    } catch (error) {
      console.error('Failed to load parquet:', error)
    }
  }

  const exportParquet = async (tableName: string) => {
    try {
      const response = await fetch(`/api/duckdb/export-parquet/${tableName}`)
      if (response.ok) {
        const blob = await response.blob()
        const url = window.URL.createObjectURL(blob)
        const link = document.createElement('a')
        link.href = url
        link.download = `${tableName}.parquet`
        document.body.appendChild(link)
        link.click()
        document.body.removeChild(link)
      }
    } catch (error) {
      console.error('Export failed:', error)
    }
  }

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
  }

  return (
    <div className="space-y-6">
      {/* Extensions Status */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Settings size={20} />
            <h3 className="text-lg font-semibold">Extensions</h3>
          </div>
        </CardHeader>
        <CardBody>
          <div className="flex flex-wrap gap-2">
            {extensions.length > 0 ? (
              extensions.map((ext) => (
                <span
                  key={ext}
                  className="px-3 py-1 bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-100 rounded-full text-sm"
                >
                  {ext}
                </span>
              ))
            ) : (
              <span className="text-gray-500 dark:text-gray-400">No extensions loaded</span>
            )}
          </div>
        </CardBody>
      </Card>

      {/* Tables List */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Database size={20} />
              <h3 className="text-lg font-semibold">Tables ({tables.length})</h3>
            </div>
            <label className="cursor-pointer">
              <Upload size={20} className="text-primary hover:text-primary-dark" />
              <input
                type="file"
                accept=".parquet"
                onChange={(e) => e.target.files?.[0] && loadParquet(e.target.files[0])}
                className="hidden"
              />
            </label>
          </div>
        </CardHeader>
        <CardBody>
          {tables.length > 0 ? (
            <div className="space-y-2">
              {tables.map((table) => (
                <div
                  key={table.name}
                  className="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg flex items-center justify-between hover:bg-gray-100 dark:hover:bg-gray-700 transition"
                >
                  <div>
                    <p className="font-medium">{table.name}</p>
                    <p className="text-sm text-gray-600 dark:text-gray-400">
                      {table.rowCount.toLocaleString()} rows • {formatBytes(table.sizeBytes)}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => queryTable(table.name)}
                    >
                      Query
                    </Button>
                    <Button
                      variant="tertiary"
                      size="sm"
                      onClick={() => exportParquet(table.name)}
                    >
                      <Download size={16} />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-gray-500 dark:text-gray-400 text-center py-4">
              No tables. Upload a Parquet file to get started.
            </p>
          )}
        </CardBody>
      </Card>

      {/* Query Editor */}
      <Card>
        <CardHeader>
          <h3 className="text-lg font-semibold">Query Editor</h3>
        </CardHeader>
        <CardBody>
          <div className="space-y-4">
            <textarea
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="SELECT * FROM table_name LIMIT 10"
              className="w-full h-32 p-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 font-mono text-sm"
            />
            <Button
              variant="primary"
              onClick={executeQuery}
              isLoading={isExecuting}
              className="w-full"
            >
              <Play size={16} />
              Execute Query
            </Button>
          </div>
        </CardBody>
      </Card>

      {/* Results */}
      {result && (
        <Card>
          <CardHeader>
            <h3 className="text-lg font-semibold">
              Results ({result.rowCount} rows in {result.executionTimeMs}ms)
            </h3>
          </CardHeader>
          <CardBody>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-gray-200 dark:border-gray-700">
                    {result.columns.map((col) => (
                      <th
                        key={col}
                        className="px-4 py-2 text-left font-medium bg-gray-50 dark:bg-gray-800"
                      >
                        {col}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {result.rows.slice(0, 100).map((row, idx) => (
                    <tr
                      key={idx}
                      className="border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800"
                    >
                      {row.map((cell, cellIdx) => (
                        <td key={cellIdx} className="px-4 py-2">
                          {cell === null ? (
                            <span className="text-gray-400">NULL</span>
                          ) : (
                            String(cell)
                          )}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </CardBody>
        </Card>
      )}
    </div>
  )
}
