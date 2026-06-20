import { useState, useRef } from 'react'
import { Upload, Download, Trash2, Cloud, Plus } from 'lucide-react'
import { Button } from './common/Button'
import { Card, CardBody, CardHeader } from './common/Card'

interface FileItem {
  id: string
  name: string
  size: number
  uploadedAt: string
  mimeType: string
}

interface CloudMount {
  id: string
  provider: string
  bucketOrPath: string
  mountPoint: string
  status: string
}

export default function FileManager({ notebookId }: { notebookId: string }) {
  const [files, setFiles] = useState<FileItem[]>([])
  const [cloudMounts, setCloudMounts] = useState<CloudMount[]>([])
  const [isUploading, setIsUploading] = useState(false)
  const [showMountForm, setShowMountForm] = useState(false)
  void setCloudMounts // Mark as used
  void setIsUploading // Mark as used
  const fileInputRef = useRef<HTMLInputElement>(null)

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = e.target.files
    if (!selectedFiles) return

    setIsUploading(true)
    for (const file of selectedFiles) {
      try {
        const formData = new FormData()
        formData.append('file', file)

        const response = await fetch(
          `/api/notebooks/${notebookId}/files/upload`,
          {
            method: 'POST',
            body: formData,
          }
        )

        if (response.ok) {
          const uploadedFile = await response.json()
          setFiles((prev) => [...prev, uploadedFile])
        }
      } catch (error) {
        console.error('Upload error:', error)
      }
    }
    setIsUploading(false)

    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }

  const handleDownload = async (fileId: string, fileName: string) => {
    try {
      const response = await fetch(
        `/api/notebooks/${notebookId}/files/${fileId}/download`
      )

      if (response.ok) {
        const blob = await response.blob()
        const url = window.URL.createObjectURL(blob)
        const link = document.createElement('a')
        link.href = url
        link.download = fileName
        document.body.appendChild(link)
        link.click()
        document.body.removeChild(link)
        window.URL.revokeObjectURL(url)
      }
    } catch (error) {
      console.error('Download error:', error)
    }
  }

  const handleDelete = async (fileId: string) => {
    try {
      const response = await fetch(
        `/api/notebooks/${notebookId}/files/${fileId}`,
        { method: 'DELETE' }
      )

      if (response.ok) {
        setFiles((prev) => prev.filter((f) => f.id !== fileId))
      }
    } catch (error) {
      console.error('Delete error:', error)
    }
  }

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
  }

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }

  return (
    <div className="space-y-6">
      {/* File Upload Section */}
      <Card>
        <CardHeader>
          <h3 className="text-lg font-semibold">File Management</h3>
        </CardHeader>
        <CardBody>
          <div className="space-y-4">
            {/* Upload Area */}
            <div
              className="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-8 text-center cursor-pointer hover:border-primary transition"
              onClick={() => fileInputRef.current?.click()}
            >
              <Upload className="mx-auto mb-3 text-gray-400" size={32} />
              <p className="text-sm font-medium mb-1">Drag and drop files or click to select</p>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                Maximum file size: 100MB
              </p>
              <input
                ref={fileInputRef}
                type="file"
                multiple
                onChange={handleFileSelect}
                className="hidden"
              />
            </div>

            {/* Files List */}
            {files.length > 0 && (
              <div className="mt-6">
                <h4 className="font-medium mb-3">Uploaded Files ({files.length})</h4>
                <div className="space-y-2">
                  {files.map((file) => (
                    <div
                      key={file.id}
                      className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition"
                    >
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium truncate">{file.name}</p>
                        <p className="text-xs text-gray-500 dark:text-gray-400">
                          {formatFileSize(file.size)} • {formatDate(file.uploadedAt)}
                        </p>
                      </div>
                      <div className="flex gap-2 ml-4">
                        <Button
                          variant="tertiary"
                          size="sm"
                          onClick={() => handleDownload(file.id, file.name)}
                          ariaLabel="Download file"
                        >
                          <Download size={16} />
                        </Button>
                        <Button
                          variant="tertiary"
                          size="sm"
                          onClick={() => handleDelete(file.id)}
                          ariaLabel="Delete file"
                        >
                          <Trash2 size={16} className="text-error" />
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </CardBody>
      </Card>

      {/* Cloud Storage Section */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Cloud size={20} />
              <h3 className="text-lg font-semibold">Cloud Storage</h3>
            </div>
            <Button
              variant="primary"
              size="sm"
              onClick={() => setShowMountForm(!showMountForm)}
            >
              <Plus size={16} />
              Mount Storage
            </Button>
          </div>
        </CardHeader>
        <CardBody>
          {showMountForm && <CloudStorageMountForm onMounted={() => setShowMountForm(false)} />}

          {cloudMounts.length > 0 && (
            <div className="space-y-2 mt-4">
              <h4 className="font-medium">Active Mounts ({cloudMounts.length})</h4>
              {cloudMounts.map((mount) => (
                <div
                  key={mount.id}
                  className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg"
                >
                  <div>
                    <p className="text-sm font-medium capitalize">{mount.provider}</p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      {mount.bucketOrPath} → {mount.mountPoint}
                    </p>
                  </div>
                  <span
                    className={`text-xs font-semibold px-2 py-1 rounded ${
                      mount.status === 'active'
                        ? 'bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-100'
                        : 'bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-100'
                    }`}
                  >
                    {mount.status}
                  </span>
                </div>
              ))}
            </div>
          )}
        </CardBody>
      </Card>
    </div>
  )
}

function CloudStorageMountForm({ onMounted }: { onMounted: () => void }) {
  const [provider, setProvider] = useState('s3')
  const [bucketPath, setBucketPath] = useState('')
  const [mountPoint, setMountPoint] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsLoading(true)

    try {
      const response = await fetch('/api/cloud-storage/mount', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          provider,
          bucketOrPath: bucketPath,
          mountPoint,
        }),
      })

      if (response.ok) {
        onMounted()
      }
    } catch (error) {
      console.error('Mount error:', error)
    }
    setIsLoading(false)
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-2">Cloud Provider</label>
        <select
          value={provider}
          onChange={(e) => setProvider(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
        >
          <option value="s3">Amazon S3</option>
          <option value="gcs">Google Cloud Storage</option>
          <option value="azure">Azure Blob Storage</option>
          <option value="gdrive">Google Drive</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">Bucket / Path</label>
        <input
          type="text"
          value={bucketPath}
          onChange={(e) => setBucketPath(e.target.value)}
          placeholder="e.g., my-bucket or /path/to/folder"
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">Mount Point</label>
        <input
          type="text"
          value={mountPoint}
          onChange={(e) => setMountPoint(e.target.value)}
          placeholder="e.g., /data or /backups"
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
        />
      </div>

      <div className="flex gap-2">
        <Button type="submit" variant="primary" isLoading={isLoading}>
          Mount Storage
        </Button>
      </div>
    </form>
  )
}
