import React, { useState } from 'react'
import { Github, Settings, Upload, Download, Check, AlertCircle } from 'lucide-react'
import { Button } from './common/Button'
import { Card, CardBody, CardHeader } from './common/Card'

interface GitHubConfig {
  id: string
  owner: string
  repo: string
  branch: string
  syncEnabled: boolean
  autoSync: boolean
  lastSync?: string
  status: string
}

interface SyncStatus {
  status: string // synced, pending, error
  lastSync?: string
  commitsAhead: number
  commitsBehind: number
}

export default function GitHubSync({ notebookId }: { notebookId: string }) {
  const [config, setConfig] = useState<GitHubConfig | null>(null)
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null)
  const [showSetup, setShowSetup] = useState(!config)
  const [isLoading, setIsLoading] = useState(false)

  const handleSetupGitHub = async (owner: string, repo: string, token: string) => {
    setIsLoading(true)
    try {
      const response = await fetch('/api/github/configure', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ owner, repo, token }),
      })

      if (response.ok) {
        const newConfig = await response.json()
        setConfig(newConfig)
        setShowSetup(false)
      }
    } catch (error) {
      console.error('Setup error:', error)
    }
    setIsLoading(false)
  }

  const handlePush = async () => {
    if (!config) return
    setIsLoading(true)
    try {
      const response = await fetch(`/api/notebooks/${notebookId}/github/push`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          commitMessage: `Update notebook at ${new Date().toISOString()}`,
        }),
      })

      if (response.ok) {
        await fetchSyncStatus()
      }
    } catch (error) {
      console.error('Push error:', error)
    }
    setIsLoading(false)
  }

  const handlePull = async () => {
    if (!config) return
    setIsLoading(true)
    try {
      const response = await fetch(`/api/notebooks/${notebookId}/github/pull`, {
        method: 'POST',
      })

      if (response.ok) {
        await fetchSyncStatus()
      }
    } catch (error) {
      console.error('Pull error:', error)
    }
    setIsLoading(false)
  }

  const fetchSyncStatus = async () => {
    if (!config) return
    try {
      const response = await fetch(`/api/notebooks/${notebookId}/github/status`)
      if (response.ok) {
        const status = await response.json()
        setSyncStatus(status)
      }
    } catch (error) {
      console.error('Status fetch error:', error)
    }
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Github size={20} />
            <h3 className="text-lg font-semibold">GitHub Sync</h3>
          </div>
          {config && (
            <Button
              variant="tertiary"
              size="sm"
              onClick={() => setShowSetup(!showSetup)}
            >
              <Settings size={16} />
            </Button>
          )}
        </div>
      </CardHeader>
      <CardBody>
        {!config || showSetup ? (
          <GitHubSetupForm onSetup={handleSetupGitHub} isLoading={isLoading} />
        ) : (
          <GitHubSyncPanel
            config={config}
            status={syncStatus}
            onPush={handlePush}
            onPull={handlePull}
            isLoading={isLoading}
          />
        )}
      </CardBody>
    </Card>
  )
}

interface GitHubSetupFormProps {
  onSetup: (owner: string, repo: string, token: string) => Promise<void>
  isLoading: boolean
}

function GitHubSetupForm({ onSetup, isLoading }: GitHubSetupFormProps) {
  const [owner, setOwner] = useState('')
  const [repo, setRepo] = useState('')
  const [token, setToken] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    await onSetup(owner, repo, token)
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-2">GitHub Owner</label>
        <input
          type="text"
          value={owner}
          onChange={(e) => setOwner(e.target.value)}
          placeholder="your-github-username"
          required
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">Repository Name</label>
        <input
          type="text"
          value={repo}
          onChange={(e) => setRepo(e.target.value)}
          placeholder="notebooks"
          required
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">GitHub Token</label>
        <input
          type="password"
          value={token}
          onChange={(e) => setToken(e.target.value)}
          placeholder="ghp_..."
          required
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
        />
        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
          Personal access token with repo scope
        </p>
      </div>

      <Button type="submit" variant="primary" isLoading={isLoading}>
        Connect GitHub
      </Button>
    </form>
  )
}

interface GitHubSyncPanelProps {
  config: GitHubConfig
  status: SyncStatus | null
  onPush: () => Promise<void>
  onPull: () => Promise<void>
  isLoading: boolean
}

function GitHubSyncPanel({
  config,
  status,
  onPush,
  onPull,
  isLoading,
}: GitHubSyncPanelProps) {
  return (
    <div className="space-y-4">
      {/* Repository Info */}
      <div className="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
        <p className="text-sm">
          <span className="font-medium">Repository:</span> {config.owner}/{config.repo}@{config.branch}
        </p>
      </div>

      {/* Sync Status */}
      {status && (
        <div
          className={`p-3 rounded-lg flex items-start gap-3 ${
            status.status === 'synced'
              ? 'bg-green-50 dark:bg-green-900 text-green-800 dark:text-green-100'
              : status.status === 'pending'
                ? 'bg-yellow-50 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-100'
                : 'bg-red-50 dark:bg-red-900 text-red-800 dark:text-red-100'
          }`}
        >
          {status.status === 'synced' ? (
            <Check size={16} className="flex-shrink-0 mt-0.5" />
          ) : (
            <AlertCircle size={16} className="flex-shrink-0 mt-0.5" />
          )}
          <div>
            <p className="font-medium text-sm capitalize">{status.status}</p>
            {status.lastSync && (
              <p className="text-xs opacity-75">
                Last sync: {new Date(status.lastSync).toLocaleString()}
              </p>
            )}
            {status.commitsAhead > 0 || status.commitsBehind > 0 ? (
              <p className="text-xs opacity-75 mt-1">
                {status.commitsAhead > 0 && `${status.commitsAhead} commits ahead`}
                {status.commitsAhead > 0 && status.commitsBehind > 0 && ' • '}
                {status.commitsBehind > 0 && `${status.commitsBehind} commits behind`}
              </p>
            ) : null}
          </div>
        </div>
      )}

      {/* Sync Actions */}
      <div className="flex gap-2">
        <Button
          variant="secondary"
          onClick={onPull}
          isLoading={isLoading}
          ariaLabel="Pull latest changes"
        >
          <Download size={16} />
          Pull
        </Button>
        <Button
          variant="primary"
          onClick={onPush}
          isLoading={isLoading}
          ariaLabel="Push changes"
        >
          <Upload size={16} />
          Push
        </Button>
      </div>

      {/* Auto-sync toggle */}
      <div className="pt-3 border-t border-gray-200 dark:border-gray-700">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={config.autoSync}
            onChange={() => {
              // TODO: Implement auto-sync toggle
            }}
            className="rounded"
          />
          <span className="text-sm">Auto-sync every 60 minutes</span>
        </label>
      </div>
    </div>
  )
}
