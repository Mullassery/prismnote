import React, { useState, useEffect } from 'react'
import { Settings, RotateCcw } from 'lucide-react'
import { Button } from './common/Button'
import { Card, CardBody, CardHeader } from './common/Card'

interface DisplaySettingsState {
  theme: string
  accentColor: string
  editorFontFamily: string
  editorFontSize: number
  editorLineHeight: number
  editorTabWidth: number
  editorWordWrap: boolean
  editorMinimap: boolean
  editorLineNumbers: boolean
  notebookCellLineLimit: number
  notebookOutputLimit: number
  notebookShowExecutionCount: boolean
  notebookAutoCollapseOutput: boolean
  notebookShowVariableTypes: boolean
  sidebarWidth: number
  sidebarVisible: boolean
  rightPanelVisible: boolean
  statusBarVisible: boolean
  tableRowsPerPage: number
  chartTheme: string
  chartAnimation: boolean
  fontScaling: number
  highContrast: boolean
  reduceMotion: boolean
}

export default function DisplaySettings() {
  const [settings, setSettings] = useState<Partial<DisplaySettingsState>>({})
  const [isLoading, setIsLoading] = useState(false)
  const [isSaved, setIsSaved] = useState(false)
  const [activeTab, setActiveTab] = useState('theme')

  useEffect(() => {
    fetchSettings()
  }, [])

  const fetchSettings = async () => {
    try {
      const response = await fetch('/api/display-settings')
      if (response.ok) {
        const data = await response.json()
        setSettings(data)
      }
    } catch (error) {
      console.error('Failed to fetch settings:', error)
    }
  }

  const handleChange = <K extends keyof DisplaySettingsState>(
    key: K,
    value: DisplaySettingsState[K]
  ) => {
    setSettings((prev) => ({ ...prev, [key]: value }))
    setIsSaved(false)
  }

  const handleSave = async () => {
    setIsLoading(true)
    try {
      const response = await fetch('/api/display-settings', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settings),
      })

      if (response.ok) {
        setIsSaved(true)
        setTimeout(() => setIsSaved(false), 3000)
      }
    } catch (error) {
      console.error('Failed to save settings:', error)
    }
    setIsLoading(false)
  }

  const handleReset = async () => {
    if (confirm('Reset all settings to defaults?')) {
      setIsLoading(true)
      try {
        const response = await fetch('/api/display-settings/reset', {
          method: 'POST',
        })

        if (response.ok) {
          await fetchSettings()
          setIsSaved(true)
          setTimeout(() => setIsSaved(false), 3000)
        }
      } catch (error) {
        console.error('Failed to reset settings:', error)
      }
      setIsLoading(false)
    }
  }

  const tabs = [
    { id: 'theme', label: 'Theme' },
    { id: 'editor', label: 'Editor' },
    { id: 'notebook', label: 'Notebook' },
    { id: 'ui', label: 'UI Layout' },
    { id: 'accessibility', label: 'Accessibility' },
  ]

  return (
    <div className="space-y-6 max-w-2xl">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Settings size={20} />
              <h2 className="text-xl font-bold">Display Settings</h2>
            </div>
            <Button
              variant="tertiary"
              size="sm"
              onClick={handleReset}
              ariaLabel="Reset to defaults"
            >
              <RotateCcw size={16} />
            </Button>
          </div>
        </CardHeader>
      </Card>

      {/* Tab Navigation */}
      <div className="flex gap-2 border-b border-gray-200 dark:border-gray-700 overflow-x-auto">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition ${
              activeTab === tab.id
                ? 'border-primary text-primary'
                : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <Card>
        <CardBody className="space-y-6">
          {activeTab === 'theme' && (
            <ThemeSettings
              settings={settings}
              onChange={handleChange}
            />
          )}

          {activeTab === 'editor' && (
            <EditorSettings
              settings={settings}
              onChange={handleChange}
            />
          )}

          {activeTab === 'notebook' && (
            <NotebookSettings
              settings={settings}
              onChange={handleChange}
            />
          )}

          {activeTab === 'ui' && (
            <UILayoutSettings
              settings={settings}
              onChange={handleChange}
            />
          )}

          {activeTab === 'accessibility' && (
            <AccessibilitySettings
              settings={settings}
              onChange={handleChange}
            />
          )}

          {/* Save Button */}
          <div className="pt-4 border-t border-gray-200 dark:border-gray-700 flex gap-2">
            <Button
              variant="primary"
              onClick={handleSave}
              isLoading={isLoading}
            >
              Save Settings
            </Button>
            {isSaved && (
              <span className="text-sm text-green-600 dark:text-green-400 flex items-center">
                ✓ Settings saved
              </span>
            )}
          </div>
        </CardBody>
      </Card>
    </div>
  )
}

function ThemeSettings({
  settings,
  onChange,
}: {
  settings: Partial<DisplaySettingsState>
  onChange: <K extends keyof DisplaySettingsState>(key: K, value: DisplaySettingsState[K]) => void
}) {
  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-2">Theme</label>
        <select
          value={settings.theme || 'auto'}
          onChange={(e) => onChange('theme', e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
        >
          <option value="light">Light</option>
          <option value="dark">Dark</option>
          <option value="auto">System Preference</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">Accent Color</label>
        <div className="flex gap-2 flex-wrap">
          {[
            { name: 'Blue', value: '#2563EB' },
            { name: 'Purple', value: '#7C3AED' },
            { name: 'Pink', value: '#EC4899' },
            { name: 'Green', value: '#10B981' },
            { name: 'Orange', value: '#F97316' },
          ].map((color) => (
            <button
              key={color.value}
              onClick={() => onChange('accentColor', color.value)}
              className={`w-10 h-10 rounded-lg border-2 transition ${
                settings.accentColor === color.value
                  ? 'border-gray-900 dark:border-white'
                  : 'border-transparent'
              }`}
              style={{ backgroundColor: color.value }}
              title={color.name}
            />
          ))}
        </div>
      </div>
    </div>
  )
}

function EditorSettings({
  settings,
  onChange,
}: {
  settings: Partial<DisplaySettingsState>
  onChange: <K extends keyof DisplaySettingsState>(key: K, value: DisplaySettingsState[K]) => void
}) {
  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-2">Font Family</label>
        <select
          value={settings.editorFontFamily || ''}
          onChange={(e) => onChange('editorFontFamily', e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 font-mono"
        >
          <option value="JetBrains Mono, Monaco, monospace">JetBrains Mono</option>
          <option value="Monaco, Menlo, monospace">Monaco</option>
          <option value="Courier New, monospace">Courier New</option>
          <option value="Consolas, monospace">Consolas</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">
          Font Size: {settings.editorFontSize}px
        </label>
        <input
          type="range"
          min="10"
          max="20"
          value={settings.editorFontSize || 14}
          onChange={(e) => onChange('editorFontSize', parseInt(e.target.value))}
          className="w-full"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">
          Line Height: {(settings.editorLineHeight || 1.6).toFixed(1)}
        </label>
        <input
          type="range"
          min="1"
          max="2"
          step="0.2"
          value={settings.editorLineHeight || 1.6}
          onChange={(e) => onChange('editorLineHeight', parseFloat(e.target.value))}
          className="w-full"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">Tab Width: {settings.editorTabWidth}</label>
        <input
          type="range"
          min="2"
          max="8"
          value={settings.editorTabWidth || 4}
          onChange={(e) => onChange('editorTabWidth', parseInt(e.target.value))}
          className="w-full"
        />
      </div>

      <div className="space-y-2">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.editorWordWrap || false}
            onChange={(e) => onChange('editorWordWrap', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">Word Wrap</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.editorMinimap || true}
            onChange={(e) => onChange('editorMinimap', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">Show Minimap</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.editorLineNumbers || true}
            onChange={(e) => onChange('editorLineNumbers', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">Show Line Numbers</span>
        </label>
      </div>
    </div>
  )
}

function NotebookSettings({
  settings,
  onChange,
}: {
  settings: Partial<DisplaySettingsState>
  onChange: <K extends keyof DisplaySettingsState>(key: K, value: DisplaySettingsState[K]) => void
}) {
  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-2">
          Cell Preview Limit: {settings.notebookCellLineLimit} lines
        </label>
        <input
          type="range"
          min="5"
          max="50"
          value={settings.notebookCellLineLimit || 20}
          onChange={(e) => onChange('notebookCellLineLimit', parseInt(e.target.value))}
          className="w-full"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">
          Output Size Limit: {settings.notebookOutputLimit}MB
        </label>
        <input
          type="range"
          min="10"
          max="500"
          step="10"
          value={settings.notebookOutputLimit || 50}
          onChange={(e) => onChange('notebookOutputLimit', parseInt(e.target.value))}
          className="w-full"
        />
      </div>

      <div className="space-y-2">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.notebookShowExecutionCount || true}
            onChange={(e) => onChange('notebookShowExecutionCount', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">Show Execution Count</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.notebookAutoCollapseOutput || false}
            onChange={(e) => onChange('notebookAutoCollapseOutput', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">Auto-collapse Long Outputs</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.notebookShowVariableTypes || true}
            onChange={(e) => onChange('notebookShowVariableTypes', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">Show Variable Types</span>
        </label>
      </div>
    </div>
  )
}

function UILayoutSettings({
  settings,
  onChange,
}: {
  settings: Partial<DisplaySettingsState>
  onChange: <K extends keyof DisplaySettingsState>(key: K, value: DisplaySettingsState[K]) => void
}) {
  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-2">
          Sidebar Width: {settings.sidebarWidth}px
        </label>
        <input
          type="range"
          min="200"
          max="400"
          value={settings.sidebarWidth || 240}
          onChange={(e) => onChange('sidebarWidth', parseInt(e.target.value))}
          className="w-full"
        />
      </div>

      <div className="space-y-2">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.sidebarVisible || true}
            onChange={(e) => onChange('sidebarVisible', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">Show Sidebar</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.rightPanelVisible || true}
            onChange={(e) => onChange('rightPanelVisible', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">Show Right Panel</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.statusBarVisible || true}
            onChange={(e) => onChange('statusBarVisible', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">Show Status Bar</span>
        </label>
      </div>
    </div>
  )
}

function AccessibilitySettings({
  settings,
  onChange,
}: {
  settings: Partial<DisplaySettingsState>
  onChange: <K extends keyof DisplaySettingsState>(key: K, value: DisplaySettingsState[K]) => void
}) {
  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-2">
          Font Scaling: {(settings.fontScaling || 1).toFixed(1)}x
        </label>
        <input
          type="range"
          min="0.8"
          max="1.5"
          step="0.1"
          value={settings.fontScaling || 1}
          onChange={(e) => onChange('fontScaling', parseFloat(e.target.value))}
          className="w-full"
        />
        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
          Zoom text across the entire application
        </p>
      </div>

      <div className="space-y-2">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.highContrast || false}
            onChange={(e) => onChange('highContrast', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">High Contrast Mode</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.reduceMotion || false}
            onChange={(e) => onChange('reduceMotion', e.target.checked)}
            className="rounded"
          />
          <span className="text-sm">Reduce Motion</span>
        </label>
      </div>
    </div>
  )
}
