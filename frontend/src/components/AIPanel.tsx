import { useState } from 'react'
import { Sparkles, Copy, AlertCircle, Loader } from 'lucide-react'
import axios from 'axios'

interface AIPanelProps {
  cellCode: string
  cellError?: string
  onInsertCode: (code: string) => void
}

export default function AIPanel({ cellCode, cellError, onInsertCode }: AIPanelProps) {
  const [action, setAction] = useState<'explain' | 'fix' | 'complete' | null>(null)
  const [response, setResponse] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [isConfigured, setIsConfigured] = useState(true)

  const getAIResponse = async (act: 'explain' | 'fix' | 'complete') => {
    if (!cellCode.trim()) {
      setError('No code to analyze')
      return
    }

    setLoading(true)
    setError('')
    setAction(act)

    try {
      const endpoint = `/api/ai/${act}`
      const payload = {
        action: act,
        code: cellCode,
        error: cellError,
        context: 'Python data science notebook',
      }

      const res = await axios.post(endpoint, payload)
      setResponse(res.data.suggestion)
    } catch (err: any) {
      if (err.response?.status === 400) {
        setIsConfigured(false)
        setError('AI not configured. Set up Ollama, Claude, or OpenAI')
      } else {
        setError(`Error: ${err.response?.data?.message || err.message}`)
      }
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="w-80 bg-slate-800 border-l border-slate-700 flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-slate-700 flex items-center gap-2">
        <Sparkles size={18} className="text-blue-400" />
        <h3 className="font-semibold text-white">AI Assistant</h3>
      </div>

      {/* Configuration Status */}
      {!isConfigured && (
        <div className="p-3 m-3 bg-amber-900 bg-opacity-30 border border-amber-700 rounded text-xs text-amber-200">
          <p className="font-semibold mb-2">Setup Required</p>
          <p className="mb-2">Configure an AI provider:</p>
          <ul className="list-disc list-inside space-y-1 text-xs">
            <li><strong>Ollama:</strong> Set PRISMNOTE_AI_PROVIDER=ollama, PRISMNOTE_OLLAMA_URL, PRISMNOTE_OLLAMA_MODEL</li>
            <li><strong>Claude:</strong> Set PRISMNOTE_AI_PROVIDER=claude, ANTHROPIC_API_KEY</li>
            <li><strong>OpenAI:</strong> Set PRISMNOTE_AI_PROVIDER=openai, OPENAI_API_KEY</li>
          </ul>
        </div>
      )}

      {/* Action Buttons */}
      <div className="p-3 space-y-2">
        <button
          onClick={() => getAIResponse('explain')}
          disabled={loading || !cellCode.trim()}
          className="w-full px-3 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 disabled:opacity-50 rounded text-sm font-medium text-white transition"
        >
          {loading && action === 'explain' ? (
            <Loader size={14} className="inline mr-2 animate-spin" />
          ) : (
            '💡'
          )}
          {' '}Explain Code
        </button>

        <button
          onClick={() => getAIResponse('fix')}
          disabled={loading || !cellCode.trim()}
          className="w-full px-3 py-2 bg-orange-600 hover:bg-orange-700 disabled:bg-slate-600 disabled:opacity-50 rounded text-sm font-medium text-white transition"
        >
          {loading && action === 'fix' ? (
            <Loader size={14} className="inline mr-2 animate-spin" />
          ) : (
            '🔧'
          )}
          {' '}Fix Error
        </button>

        <button
          onClick={() => getAIResponse('complete')}
          disabled={loading || !cellCode.trim()}
          className="w-full px-3 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-slate-600 disabled:opacity-50 rounded text-sm font-medium text-white transition"
        >
          {loading && action === 'complete' ? (
            <Loader size={14} className="inline mr-2 animate-spin" />
          ) : (
            '✨'
          )}
          {' '}Complete
        </button>
      </div>

      {/* Response */}
      {response && (
        <div className="flex-1 overflow-y-auto p-3 space-y-2">
          <div className="bg-slate-700 p-3 rounded text-sm text-gray-100">
            <pre className="font-mono text-xs whitespace-pre-wrap break-words">{response}</pre>
          </div>

          <button
            onClick={() => onInsertCode(response)}
            className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-green-600 hover:bg-green-700 rounded text-sm text-white font-medium transition"
          >
            <Copy size={14} />
            Insert Code
          </button>

          <button
            onClick={() => setResponse('')}
            className="w-full px-3 py-2 bg-slate-700 hover:bg-slate-600 rounded text-xs text-gray-300 transition"
          >
            Clear
          </button>
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="p-3 m-3 bg-red-900 bg-opacity-30 border border-red-700 rounded text-xs text-red-200 flex gap-2">
          <AlertCircle size={14} className="flex-shrink-0 mt-0.5" />
          <p>{error}</p>
        </div>
      )}

      {/* Empty State */}
      {!response && !error && !loading && (
        <div className="flex-1 flex items-center justify-center p-3 text-center">
          <p className="text-xs text-gray-500">
            Select code in a cell and click an action to get AI assistance
          </p>
        </div>
      )}
    </div>
  )
}
