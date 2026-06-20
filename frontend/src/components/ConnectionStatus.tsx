import { useState, useEffect } from 'react'
import { Activity, Database, Cloud, AlertCircle, Check, X, Clock } from 'lucide-react'
import { Card, CardBody, CardHeader } from './common/Card'

interface Connection {
  id: string
  name: string
  connectionType: string
  provider: string
  status: string
  latencyMs?: number
  lastChecked: string
  errorMessage?: string
  stats: ConnectionStats
}

interface ConnectionStats {
  totalConnections: number
  activeQueries: number
  queriesRun: number
  totalBytesTransferred: number
  uptimeSeconds: number
}

interface ConnectionsOverview {
  totalConnections: number
  connected: number
  disconnected: number
  errorCount: number
  connections: Connection[]
  lastRefresh: string
}

export default function ConnectionStatus() {
  const [overview, setOverview] = useState<ConnectionsOverview | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [selectedConnection, setSelectedConnection] = useState<Connection | null>(null)
  const [autoRefresh, setAutoRefresh] = useState(true)

  useEffect(() => {
    fetchConnections()
    if (autoRefresh) {
      const interval = setInterval(fetchConnections, 30000) // Refresh every 30s
      return () => clearInterval(interval)
    }
  }, [autoRefresh])

  const fetchConnections = async () => {
    try {
      const response = await fetch('/api/connections/overview')
      if (response.ok) {
        const data = await response.json()
        setOverview(data)
      }
    } catch (error) {
      console.error('Failed to fetch connections:', error)
    }
    setIsLoading(false)
  }

  const refreshConnection = async (connectionId: string) => {
    try {
      const response = await fetch(`/api/connections/${connectionId}/health-check`, {
        method: 'POST',
      })
      if (response.ok) {
        await fetchConnections()
      }
    } catch (error) {
      console.error('Health check failed:', error)
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'connected':
        return <Check size={16} className="text-success" />
      case 'disconnected':
        return <X size={16} className="text-gray-400" />
      case 'error':
        return <AlertCircle size={16} className="text-error" />
      case 'connecting':
        return <Clock size={16} className="text-warning animate-spin" />
      default:
        return <AlertCircle size={16} className="text-gray-400" />
    }
  }

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'database':
        return <Database size={20} />
      case 'data_warehouse':
        return <Cloud size={20} />
      case 'file_storage':
        return <Cloud size={20} />
      case 'duckdb':
        return <Database size={20} />
      case 'iceberg':
        return <Database size={20} />
      default:
        return <Activity size={20} />
    }
  }

  const formatLatency = (ms?: number) => {
    if (!ms) return 'N/A'
    return `${ms}ms`
  }

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
  }

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)

    if (days > 0) return `${days}d ${hours}h`
    if (hours > 0) return `${hours}h ${minutes}m`
    return `${minutes}m`
  }

  return (
    <div className="space-y-6">
      {/* Overview Cards */}
      {overview && (
        <div className="grid grid-cols-4 gap-4">
          <Card className="text-center">
            <CardBody>
              <div className="text-3xl font-bold text-primary mb-2">
                {overview.totalConnections}
              </div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Total Connections</p>
            </CardBody>
          </Card>
          <Card className="text-center">
            <CardBody>
              <div className="text-3xl font-bold text-success mb-2">{overview.connected}</div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Connected</p>
            </CardBody>
          </Card>
          <Card className="text-center">
            <CardBody>
              <div className="text-3xl font-bold text-gray-500 mb-2">
                {overview.disconnected}
              </div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Disconnected</p>
            </CardBody>
          </Card>
          <Card className="text-center">
            <CardBody>
              <div className="text-3xl font-bold text-error mb-2">{overview.errorCount}</div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Errors</p>
            </CardBody>
          </Card>
        </div>
      )}

      {/* Main Connections List */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Activity size={20} />
              <h3 className="text-lg font-semibold">External Connections</h3>
            </div>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
                className="rounded"
              />
              <span className="text-sm">Auto-refresh</span>
            </label>
          </div>
        </CardHeader>
        <CardBody>
          {isLoading ? (
            <div className="text-center py-8">
              <p className="text-gray-500 dark:text-gray-400">Loading connections...</p>
            </div>
          ) : overview && overview.connections.length > 0 ? (
            <div className="space-y-3">
              {overview.connections.map((connection) => (
                <div
                  key={connection.id}
                  className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition cursor-pointer"
                  onClick={() => setSelectedConnection(connection)}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-start gap-3 flex-1">
                      <div className="mt-1 text-gray-600 dark:text-gray-400">
                        {getTypeIcon(connection.connectionType)}
                      </div>
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <h4 className="font-medium">{connection.name}</h4>
                          <span className="text-xs px-2 py-1 rounded-full bg-gray-100 dark:bg-gray-700">
                            {connection.provider}
                          </span>
                        </div>
                        <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                          {connection.connectionType.replace('_', ' ')}
                        </p>
                        <div className="flex gap-4 text-xs text-gray-600 dark:text-gray-400">
                          <span>Latency: {formatLatency(connection.latencyMs)}</span>
                          <span>Queries: {connection.stats.queriesRun}</span>
                          <span>Data: {formatBytes(connection.stats.totalBytesTransferred)}</span>
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center gap-3 ml-4">
                      <div className="text-right">
                        <div
                          className={`text-sm font-medium capitalize flex items-center gap-1 justify-end ${
                            connection.status === 'connected'
                              ? 'text-success'
                              : connection.status === 'error'
                                ? 'text-error'
                                : 'text-gray-500'
                          }`}
                        >
                          {getStatusIcon(connection.status)}
                          {connection.status}
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                          Checked {formatTimeAgo(connection.lastChecked)}
                        </p>
                      </div>
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          refreshConnection(connection.id)
                        }}
                        className="px-2 py-1 text-xs bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 rounded transition"
                      >
                        Check
                      </button>
                    </div>
                  </div>

                  {connection.errorMessage && (
                    <div className="mt-3 p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900 rounded text-sm text-red-700 dark:text-red-400">
                      {connection.errorMessage}
                    </div>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8">
              <p className="text-gray-500 dark:text-gray-400">No connections configured</p>
            </div>
          )}
        </CardBody>
      </Card>

      {/* Connection Details */}
      {selectedConnection && (
        <Card>
          <CardHeader>
            <h3 className="text-lg font-semibold">{selectedConnection.name} - Details</h3>
          </CardHeader>
          <CardBody>
            <div className="grid grid-cols-2 gap-6">
              <div>
                <h4 className="font-medium mb-3">Connection Info</h4>
                <dl className="space-y-2 text-sm">
                  <div>
                    <dt className="text-gray-600 dark:text-gray-400">Type</dt>
                    <dd className="font-medium capitalize">{selectedConnection.connectionType}</dd>
                  </div>
                  <div>
                    <dt className="text-gray-600 dark:text-gray-400">Provider</dt>
                    <dd className="font-medium">{selectedConnection.provider}</dd>
                  </div>
                  <div>
                    <dt className="text-gray-600 dark:text-gray-400">Status</dt>
                    <dd className="font-medium flex items-center gap-2">
                      {getStatusIcon(selectedConnection.status)}
                      {selectedConnection.status}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-gray-600 dark:text-gray-400">Latency</dt>
                    <dd className="font-medium">{formatLatency(selectedConnection.latencyMs)}</dd>
                  </div>
                </dl>
              </div>
              <div>
                <h4 className="font-medium mb-3">Statistics</h4>
                <dl className="space-y-2 text-sm">
                  <div>
                    <dt className="text-gray-600 dark:text-gray-400">Queries Run</dt>
                    <dd className="font-medium">{selectedConnection.stats.queriesRun}</dd>
                  </div>
                  <div>
                    <dt className="text-gray-600 dark:text-gray-400">Active Queries</dt>
                    <dd className="font-medium">{selectedConnection.stats.activeQueries}</dd>
                  </div>
                  <div>
                    <dt className="text-gray-600 dark:text-gray-400">Data Transferred</dt>
                    <dd className="font-medium">
                      {formatBytes(selectedConnection.stats.totalBytesTransferred)}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-gray-600 dark:text-gray-400">Uptime</dt>
                    <dd className="font-medium">
                      {formatUptime(selectedConnection.stats.uptimeSeconds)}
                    </dd>
                  </div>
                </dl>
              </div>
            </div>
          </CardBody>
        </Card>
      )}
    </div>
  )
}

function formatTimeAgo(timestamp: string) {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffSecs = Math.floor(diffMs / 1000)
  const diffMins = Math.floor(diffSecs / 60)
  const diffHours = Math.floor(diffMins / 60)

  if (diffSecs < 60) return 'just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  return `${Math.floor(diffHours / 24)}d ago`
}
