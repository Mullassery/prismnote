import { useState } from 'react'
import { Database, Plus, Trash2, TestTube, Code } from 'lucide-react'

interface DatabaseConnection {
  id: string
  name: string
  db_type: string
  host?: string
  port?: number
  database: string
  created_at: string
}

export default function DatabaseConnector() {
  const [databases, setDatabases] = useState<DatabaseConnection[]>([])
  const [showForm, setShowForm] = useState(false)
  const [formData, setFormData] = useState({
    name: '',
    db_type: 'postgresql',
    host: 'localhost',
    port: '5432',
    database: '',
    username: '',
    password: '',
  })

  const handleAddDatabase = async () => {
    if (!formData.name || !formData.database) {
      alert('Name and database are required')
      return
    }

    const newDb: DatabaseConnection = {
      id: Math.random().toString(36),
      name: formData.name,
      db_type: formData.db_type,
      host: formData.host || undefined,
      port: formData.port ? parseInt(formData.port) : undefined,
      database: formData.database,
      created_at: new Date().toISOString(),
    }

    setDatabases([...databases, newDb])
    setFormData({
      name: '',
      db_type: 'postgresql',
      host: 'localhost',
      port: '5432',
      database: '',
      username: '',
      password: '',
    })
    setShowForm(false)
  }

  const handleTestConnection = async (db: DatabaseConnection) => {
    try {
      const res = await fetch(`/api/databases/${db.id}/test`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(db),
      })
      const data = await res.json()
      alert(data.message || 'Connection test started')
    } catch (err) {
      alert(`Error: ${err}`)
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-white flex items-center gap-2">
          <Database size={20} />
          Database Connections
        </h2>
        <button
          onClick={() => setShowForm(!showForm)}
          className="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-sm text-white flex items-center gap-2"
        >
          <Plus size={16} />
          Add Database
        </button>
      </div>

      {/* Form */}
      {showForm && (
        <div className="p-4 bg-slate-800 rounded border border-slate-700 space-y-3">
          <div>
            <label className="block text-xs font-medium text-gray-300 mb-1">
              Name
            </label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              placeholder="My Database"
              className="w-full px-2 py-1 bg-slate-700 border border-slate-600 rounded text-xs text-white"
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-gray-300 mb-1">
              Database Type
            </label>
            <select
              value={formData.db_type}
              onChange={(e) => setFormData({ ...formData, db_type: e.target.value })}
              className="w-full px-2 py-1 bg-slate-700 border border-slate-600 rounded text-xs text-white"
            >
              <option>postgresql</option>
              <option>mysql</option>
              <option>sqlite</option>
              <option>duckdb</option>
              <option>mongodb</option>
            </select>
          </div>

          {formData.db_type !== 'mongodb' && (
            <>
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-xs font-medium text-gray-300 mb-1">
                    Host
                  </label>
                  <input
                    type="text"
                    value={formData.host}
                    onChange={(e) => setFormData({ ...formData, host: e.target.value })}
                    placeholder="localhost"
                    className="w-full px-2 py-1 bg-slate-700 border border-slate-600 rounded text-xs text-white"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-gray-300 mb-1">
                    Port
                  </label>
                  <input
                    type="number"
                    value={formData.port}
                    onChange={(e) => setFormData({ ...formData, port: e.target.value })}
                    className="w-full px-2 py-1 bg-slate-700 border border-slate-600 rounded text-xs text-white"
                  />
                </div>
              </div>
            </>
          )}

          <div>
            <label className="block text-xs font-medium text-gray-300 mb-1">
              Database
            </label>
            <input
              type="text"
              value={formData.database}
              onChange={(e) => setFormData({ ...formData, database: e.target.value })}
              placeholder="database_name"
              className="w-full px-2 py-1 bg-slate-700 border border-slate-600 rounded text-xs text-white"
            />
          </div>

          {formData.db_type !== 'sqlite' && formData.db_type !== 'duckdb' && (
            <>
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-xs font-medium text-gray-300 mb-1">
                    Username
                  </label>
                  <input
                    type="text"
                    value={formData.username}
                    onChange={(e) => setFormData({ ...formData, username: e.target.value })}
                    className="w-full px-2 py-1 bg-slate-700 border border-slate-600 rounded text-xs text-white"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-gray-300 mb-1">
                    Password
                  </label>
                  <input
                    type="password"
                    value={formData.password}
                    onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                    className="w-full px-2 py-1 bg-slate-700 border border-slate-600 rounded text-xs text-white"
                  />
                </div>
              </div>
            </>
          )}

          <div className="flex gap-2">
            <button
              onClick={handleAddDatabase}
              className="flex-1 px-3 py-2 bg-green-600 hover:bg-green-700 rounded text-xs text-white"
            >
              Add Connection
            </button>
            <button
              onClick={() => setShowForm(false)}
              className="flex-1 px-3 py-2 bg-slate-700 hover:bg-slate-600 rounded text-xs text-white"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Database List */}
      <div className="space-y-2">
        {databases.length === 0 ? (
          <p className="text-xs text-gray-500 p-4 text-center">
            No databases connected. Add one to write SQL queries in notebooks.
          </p>
        ) : (
          databases.map((db) => (
            <div
              key={db.id}
              className="p-3 bg-slate-800 rounded border border-slate-700 space-y-2"
            >
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-semibold text-white text-sm">{db.name}</p>
                  <p className="text-xs text-gray-400">
                    {db.db_type} • {db.database}
                    {db.host && ` @ ${db.host}:${db.port || 'default'}`}
                  </p>
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => handleTestConnection(db)}
                    className="p-1 hover:bg-slate-700 rounded transition"
                    title="Test connection"
                  >
                    <TestTube size={14} className="text-blue-400" />
                  </button>
                  <button
                    onClick={() =>
                      setDatabases(databases.filter((d) => d.id !== db.id))
                    }
                    className="p-1 hover:bg-red-900 rounded transition"
                    title="Delete connection"
                  >
                    <Trash2 size={14} className="text-red-400" />
                  </button>
                </div>
              </div>

              <p className="text-xs text-gray-500">
                In a SQL cell, use: <code className="bg-slate-700 px-1 rounded">
                  SELECT * FROM table_name
                </code>
              </p>
            </div>
          ))
        )}
      </div>

      {/* Info */}
      <div className="p-3 bg-slate-800 rounded border border-slate-700 text-xs text-gray-400 space-y-1">
        <p className="font-semibold text-gray-300 flex items-center gap-2">
          <Code size={14} />
          Supported Databases (OSS)
        </p>
        <ul className="list-disc list-inside space-y-1 ml-1">
          <li>
            <strong>PostgreSQL</strong> — Requires: <code>pip install psycopg2-binary</code>
          </li>
          <li>
            <strong>MySQL</strong> — Requires: <code>pip install mysql-connector-python</code>
          </li>
          <li>
            <strong>SQLite</strong> — Built-in (no install needed)
          </li>
          <li>
            <strong>DuckDB</strong> — Requires: <code>pip install duckdb</code>
          </li>
          <li>
            <strong>MongoDB</strong> — Requires: <code>pip install pymongo</code>
          </li>
        </ul>
      </div>
    </div>
  )
}
