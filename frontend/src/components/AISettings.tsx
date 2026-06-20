import { useState } from 'react'
import { Settings, X } from 'lucide-react'

interface AISettingsProps {
  isOpen: boolean
  onClose: () => void
}

export default function AISettings({ isOpen, onClose }: AISettingsProps) {
  const [provider, setProvider] = useState<'ollama' | 'claude' | 'openai'>('ollama')
  const [ollamaUrl, setOllamaUrl] = useState('http://localhost:11434')
  const [ollamaModel, setOllamaModel] = useState('neural-chat')
  const [claudeKey, setClaudeKey] = useState('')
  const [openaiKey, setOpenaiKey] = useState('')
  const [openaiModel, setOpenaiModel] = useState('gpt-4')

  const handleSave = () => {
    // Save configuration (would persist to backend/localStorage)
    console.log({
      provider,
      ollamaUrl,
      ollamaModel,
      claudeKey: claudeKey ? '***' : '',
      openaiKey: openaiKey ? '***' : '',
      openaiModel,
    })
    onClose()
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-slate-800 rounded-lg w-96 max-h-96 overflow-y-auto border border-slate-700">
        {/* Header */}
        <div className="p-4 border-b border-slate-700 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Settings size={18} />
            <h2 className="font-semibold text-white">AI Settings</h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 hover:bg-slate-700 rounded transition"
          >
            <X size={18} />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-4">
          {/* Provider Selection */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              AI Provider
            </label>
            <div className="space-y-2">
              {(['ollama', 'claude', 'openai'] as const).map((p) => (
                <label key={p} className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    value={p}
                    checked={provider === p}
                    onChange={(e) => setProvider(e.target.value as typeof p)}
                    className="w-4 h-4"
                  />
                  <span className="text-sm text-gray-300 capitalize">{p}</span>
                </label>
              ))}
            </div>
          </div>

          {/* Ollama */}
          {provider === 'ollama' && (
            <div className="space-y-3 p-3 bg-slate-700 rounded">
              <div>
                <label className="block text-xs font-medium text-gray-300 mb-1">
                  Ollama URL
                </label>
                <input
                  type="text"
                  value={ollamaUrl}
                  onChange={(e) => setOllamaUrl(e.target.value)}
                  placeholder="http://localhost:11434"
                  className="w-full px-2 py-1 bg-slate-600 border border-slate-500 rounded text-xs text-white"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-gray-300 mb-1">
                  Model Name
                </label>
                <input
                  type="text"
                  value={ollamaModel}
                  onChange={(e) => setOllamaModel(e.target.value)}
                  placeholder="neural-chat, llama2, mistral..."
                  className="w-full px-2 py-1 bg-slate-600 border border-slate-500 rounded text-xs text-white"
                />
              </div>
              <p className="text-xs text-gray-400">
                Tip: Run Ollama locally with <code className="bg-slate-600 px-1 rounded">ollama pull neural-chat</code>
              </p>
            </div>
          )}

          {/* Claude */}
          {provider === 'claude' && (
            <div className="space-y-2 p-3 bg-slate-700 rounded">
              <label className="block text-xs font-medium text-gray-300 mb-1">
                Anthropic API Key
              </label>
              <input
                type="password"
                value={claudeKey}
                onChange={(e) => setClaudeKey(e.target.value)}
                placeholder="sk-ant-..."
                className="w-full px-2 py-1 bg-slate-600 border border-slate-500 rounded text-xs text-white"
              />
              <p className="text-xs text-gray-400">
                Get key at <a href="https://console.anthropic.com" target="_blank" rel="noopener noreferrer" className="text-blue-400 hover:underline">console.anthropic.com</a>
              </p>
            </div>
          )}

          {/* OpenAI */}
          {provider === 'openai' && (
            <div className="space-y-3 p-3 bg-slate-700 rounded">
              <div>
                <label className="block text-xs font-medium text-gray-300 mb-1">
                  OpenAI API Key
                </label>
                <input
                  type="password"
                  value={openaiKey}
                  onChange={(e) => setOpenaiKey(e.target.value)}
                  placeholder="sk-..."
                  className="w-full px-2 py-1 bg-slate-600 border border-slate-500 rounded text-xs text-white"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-gray-300 mb-1">
                  Model
                </label>
                <input
                  type="text"
                  value={openaiModel}
                  onChange={(e) => setOpenaiModel(e.target.value)}
                  placeholder="gpt-4, gpt-3.5-turbo..."
                  className="w-full px-2 py-1 bg-slate-600 border border-slate-500 rounded text-xs text-white"
                />
              </div>
              <p className="text-xs text-gray-400">
                Get key at <a href="https://platform.openai.com/api-keys" target="_blank" rel="noopener noreferrer" className="text-blue-400 hover:underline">platform.openai.com</a>
              </p>
            </div>
          )}

          {/* Buttons */}
          <div className="flex gap-2 pt-4 border-t border-slate-700">
            <button
              onClick={onClose}
              className="flex-1 px-3 py-2 bg-slate-700 hover:bg-slate-600 rounded text-sm text-white transition"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              className="flex-1 px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm text-white font-medium transition"
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
