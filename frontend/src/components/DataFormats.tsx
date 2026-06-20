import { useState, useEffect } from 'react'
import { FileText, Info, BookOpen } from 'lucide-react'
import { Card, CardBody, CardHeader } from './common/Card'

interface FileFormatInfo {
  format: string
  description: string
  compressionFormats: string[]
  useCases: string[]
  advantages: string[]
  disadvantages: string[]
  ecosystemSupport: string[]
}

interface IcebergTable {
  tableId: string
  database: string
  tableName: string
  rowCount: number
  fileCount: number
  sizeBytes: number
  createdAt: string
  snapshots: Array<any>
}

export default function DataFormats() {
  const [formats, setFormats] = useState<FileFormatInfo[]>([])
  const [selectedFormat, setSelectedFormat] = useState<FileFormatInfo | null>(null)
  const [icebergTables, setIcebergTables] = useState<IcebergTable[]>([])
  const [activeTab, setActiveTab] = useState<'formats' | 'iceberg'>('formats')

  useEffect(() => {
    loadFormats()
    loadIcebergTables()
  }, [])

  const loadFormats = async () => {
    try {
      const response = await fetch('/api/file-formats')
      if (response.ok) {
        const data = await response.json()
        setFormats(data)
        if (data.length > 0) {
          setSelectedFormat(data[0])
        }
      }
    } catch (error) {
      console.error('Failed to load formats:', error)
    }
  }

  const loadIcebergTables = async () => {
    try {
      const response = await fetch('/api/iceberg/tables')
      if (response.ok) {
        const data = await response.json()
        setIcebergTables(data)
      }
    } catch (error) {
      console.error('Failed to load Iceberg tables:', error)
    }
  }

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
  }

  return (
    <div className="space-y-6">
      {/* Tab Navigation */}
      <div className="flex gap-2 border-b border-gray-200 dark:border-gray-700">
        <button
          onClick={() => setActiveTab('formats')}
          className={`px-4 py-2 font-medium border-b-2 transition ${
            activeTab === 'formats'
              ? 'border-primary text-primary'
              : 'border-transparent text-gray-600 dark:text-gray-400'
          }`}
        >
          File Formats
        </button>
        <button
          onClick={() => setActiveTab('iceberg')}
          className={`px-4 py-2 font-medium border-b-2 transition ${
            activeTab === 'iceberg'
              ? 'border-primary text-primary'
              : 'border-transparent text-gray-600 dark:text-gray-400'
          }`}
        >
          Apache Iceberg
        </button>
      </div>

      {/* File Formats Tab */}
      {activeTab === 'formats' && (
        <div className="grid grid-cols-3 gap-6">
          {/* Format List */}
          <Card className="col-span-1">
            <CardHeader>
              <div className="flex items-center gap-2">
                <FileText size={20} />
                <h3 className="text-lg font-semibold">Formats</h3>
              </div>
            </CardHeader>
            <CardBody>
              <div className="space-y-2">
                {formats.map((fmt) => (
                  <button
                    key={fmt.format}
                    onClick={() => setSelectedFormat(fmt)}
                    className={`w-full text-left px-3 py-2 rounded-lg transition ${
                      selectedFormat?.format === fmt.format
                        ? 'bg-primary text-white'
                        : 'hover:bg-gray-100 dark:hover:bg-gray-800'
                    }`}
                  >
                    {fmt.format}
                  </button>
                ))}
              </div>
            </CardBody>
          </Card>

          {/* Format Details */}
          <Card className="col-span-2">
            {selectedFormat ? (
              <CardBody className="space-y-6">
                <div>
                  <h3 className="text-xl font-bold mb-2">{selectedFormat.format}</h3>
                  <p className="text-gray-600 dark:text-gray-400">{selectedFormat.description}</p>
                </div>

                <div>
                  <h4 className="font-semibold mb-2">Use Cases</h4>
                  <ul className="list-disc list-inside space-y-1">
                    {selectedFormat.useCases.map((use) => (
                      <li key={use} className="text-sm text-gray-700 dark:text-gray-300">
                        {use}
                      </li>
                    ))}
                  </ul>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <h4 className="font-semibold mb-2 text-success">Advantages</h4>
                    <ul className="space-y-1">
                      {selectedFormat.advantages.map((adv) => (
                        <li key={adv} className="text-sm text-gray-700 dark:text-gray-300">
                          ✓ {adv}
                        </li>
                      ))}
                    </ul>
                  </div>
                  <div>
                    <h4 className="font-semibold mb-2 text-error">Disadvantages</h4>
                    <ul className="space-y-1">
                      {selectedFormat.disadvantages.map((dis) => (
                        <li key={dis} className="text-sm text-gray-700 dark:text-gray-300">
                          ✗ {dis}
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>

                <div>
                  <h4 className="font-semibold mb-2">Compression Formats</h4>
                  <div className="flex flex-wrap gap-2">
                    {selectedFormat.compressionFormats.map((codec) => (
                      <span
                        key={codec}
                        className="px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded text-sm"
                      >
                        {codec}
                      </span>
                    ))}
                  </div>
                </div>

                <div>
                  <h4 className="font-semibold mb-2">Ecosystem Support</h4>
                  <div className="flex flex-wrap gap-2">
                    {selectedFormat.ecosystemSupport.map((tool) => (
                      <span
                        key={tool}
                        className="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-100 rounded-full text-sm"
                      >
                        {tool}
                      </span>
                    ))}
                  </div>
                </div>
              </CardBody>
            ) : (
              <CardBody>
                <p className="text-gray-500 dark:text-gray-400 text-center">
                  Select a format to view details
                </p>
              </CardBody>
            )}
          </Card>
        </div>
      )}

      {/* Apache Iceberg Tab */}
      {activeTab === 'iceberg' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <div className="flex items-center gap-2">
                <BookOpen size={20} />
                <h3 className="text-lg font-semibold">Apache Iceberg Tables</h3>
              </div>
            </CardHeader>
            <CardBody>
              {icebergTables.length > 0 ? (
                <div className="overflow-x-auto">
                  <table className="w-full text-sm">
                    <thead>
                      <tr className="border-b border-gray-200 dark:border-gray-700">
                        <th className="px-4 py-2 text-left font-semibold">Table Name</th>
                        <th className="px-4 py-2 text-left font-semibold">Database</th>
                        <th className="px-4 py-2 text-left font-semibold">Rows</th>
                        <th className="px-4 py-2 text-left font-semibold">Files</th>
                        <th className="px-4 py-2 text-left font-semibold">Size</th>
                        <th className="px-4 py-2 text-left font-semibold">Snapshots</th>
                        <th className="px-4 py-2 text-left font-semibold">Created</th>
                      </tr>
                    </thead>
                    <tbody>
                      {icebergTables.map((table) => (
                        <tr
                          key={table.tableId}
                          className="border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800"
                        >
                          <td className="px-4 py-2 font-medium">{table.tableName}</td>
                          <td className="px-4 py-2">{table.database}</td>
                          <td className="px-4 py-2">{table.rowCount.toLocaleString()}</td>
                          <td className="px-4 py-2">{table.fileCount}</td>
                          <td className="px-4 py-2">{formatBytes(table.sizeBytes)}</td>
                          <td className="px-4 py-2">{table.snapshots.length}</td>
                          <td className="px-4 py-2 text-xs">
                            {new Date(table.createdAt).toLocaleDateString()}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : (
                <p className="text-gray-500 dark:text-gray-400 text-center py-8">
                  No Iceberg tables configured
                </p>
              )}
            </CardBody>
          </Card>

          <Card>
            <CardHeader>
              <div className="flex items-center gap-2">
                <Info size={20} />
                <h3 className="text-lg font-semibold">About Apache Iceberg</h3>
              </div>
            </CardHeader>
            <CardBody className="space-y-4">
              <p className="text-gray-700 dark:text-gray-300">
                Apache Iceberg is an open table format for huge analytic tables. It brings the
                reliability and simplicity of SQL tables to big data, while making it possible for
                SQL engines like Spark, Trino, Flink, and Hive to safely work with the same tables.
              </p>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <h4 className="font-semibold mb-2">Key Features</h4>
                  <ul className="space-y-1 text-sm">
                    <li>✓ ACID guarantees</li>
                    <li>✓ Schema evolution</li>
                    <li>✓ Hidden partitioning</li>
                    <li>✓ Partition evolution</li>
                    <li>✓ Concurrent writes</li>
                    <li>✓ Time travel queries</li>
                  </ul>
                </div>
                <div>
                  <h4 className="font-semibold mb-2">Supported Tools</h4>
                  <ul className="space-y-1 text-sm">
                    <li>• Apache Spark</li>
                    <li>• Apache Flink</li>
                    <li>• Trino / Presto</li>
                    <li>• DuckDB</li>
                    <li>• Dask</li>
                    <li>• Pandas</li>
                  </ul>
                </div>
              </div>
            </CardBody>
          </Card>
        </div>
      )}
    </div>
  )
}
