import { useState } from 'react'
import { BookOpen, Zap, TrendingUp, Package, X, Check } from 'lucide-react'

interface LibrarySuggestion {
  name: string
  version: string
  description: string
  reasoning: string
  installed_version?: string
  is_update: boolean
  category: 'data' | 'viz' | 'ml' | 'web' | 'utility'
  confidence: number
}

interface LibrarySuggesterProps {
  suggestions: LibrarySuggestion[]
  onInstall: (name: string, version?: string) => void
  onIgnore: (name: string) => void
  isLoading: boolean
  detectedIntent?: string
  contextSummary?: string
}

const getCategoryIcon = (category: string) => {
  switch (category) {
    case 'data':
      return <TrendingUp size={16} className="text-blue-400" />
    case 'viz':
      return <Zap size={16} className="text-purple-400" />
    case 'ml':
      return <BookOpen size={16} className="text-orange-400" />
    case 'web':
      return <Package size={16} className="text-green-400" />
    default:
      return <Package size={16} className="text-gray-400" />
  }
}

const getCategoryLabel = (category: string) => {
  return category.charAt(0).toUpperCase() + category.slice(1).replace('_', ' ')
}

export default function LibrarySuggester({
  suggestions,
  onInstall,
  onIgnore,
  isLoading,
  detectedIntent = 'Analyzing code...',
  contextSummary = '',
}: LibrarySuggesterProps) {
  const [expandedSuggestion, setExpandedSuggestion] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'all' | 'new' | 'updates'>('all')
  const [ignoredLocally, setIgnoredLocally] = useState<Set<string>>(new Set())

  const updates = suggestions.filter((s) => s.is_update)
  const newLibs = suggestions.filter((s) => !s.installed_version)
  const allVisible = suggestions.filter((s) => !ignoredLocally.has(s.name))

  const displayedSuggestions =
    activeTab === 'all'
      ? allVisible
      : activeTab === 'updates'
        ? updates.filter((s) => !ignoredLocally.has(s.name))
        : newLibs.filter((s) => !ignoredLocally.has(s.name))

  const handleIgnore = (name: string) => {
    setIgnoredLocally((prev) => new Set(prev).add(name))
    onIgnore(name)
  }

  return (
    <div className="bg-slate-800 rounded border border-slate-700 h-full flex flex-col overflow-hidden">
      {/* Header */}
      <div className="p-3 border-b border-slate-700 flex items-center gap-2">
        <BookOpen size={16} className="text-blue-400" />
        <div className="flex-1">
          <h3 className="font-semibold text-white text-sm">Library Recommendations</h3>
          <p className="text-xs text-gray-400">AI-powered discovery</p>
        </div>
        {isLoading && <div className="w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin" />}
      </div>

      {/* Intent Summary */}
      {detectedIntent && (
        <div className="px-3 py-2 border-b border-slate-700 bg-slate-700/30">
          <p className="text-xs text-gray-300">
            <span className="font-semibold text-blue-300">Context:</span> {detectedIntent}
          </p>
          {contextSummary && <p className="text-xs text-gray-400 mt-1">{contextSummary}</p>}
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-0 border-b border-slate-700 px-2">
        <button
          onClick={() => setActiveTab('all')}
          className={`px-3 py-2 text-xs font-medium border-b-2 transition ${
            activeTab === 'all'
              ? 'border-blue-400 text-blue-400'
              : 'border-transparent text-gray-400 hover:text-white'
          }`}
        >
          All ({allVisible.length})
        </button>
        {updates.length > 0 && (
          <button
            onClick={() => setActiveTab('updates')}
            className={`px-3 py-2 text-xs font-medium border-b-2 transition ${
              activeTab === 'updates'
                ? 'border-green-400 text-green-400'
                : 'border-transparent text-gray-400 hover:text-white'
            }`}
          >
            Updates ({updates.filter((s) => !ignoredLocally.has(s.name)).length})
          </button>
        )}
        {newLibs.length > 0 && (
          <button
            onClick={() => setActiveTab('new')}
            className={`px-3 py-2 text-xs font-medium border-b-2 transition ${
              activeTab === 'new'
                ? 'border-purple-400 text-purple-400'
                : 'border-transparent text-gray-400 hover:text-white'
            }`}
          >
            New ({newLibs.filter((s) => !ignoredLocally.has(s.name)).length})
          </button>
        )}
      </div>

      {/* Suggestions List */}
      <div className="flex-1 overflow-y-auto p-2 space-y-2">
        {displayedSuggestions.length === 0 ? (
          <div className="text-center py-8">
            <p className="text-xs text-gray-500">
              {ignoredLocally.size > 0
                ? 'All recommendations ignored. Keep coding to see new suggestions!'
                : 'No suggestions yet. Write some code and we\'ll suggest helpful libraries.'}
            </p>
          </div>
        ) : (
          displayedSuggestions.map((suggestion) => (
            <div
              key={suggestion.name}
              className="bg-slate-700 rounded border border-slate-600 overflow-hidden hover:border-slate-500 transition"
            >
              {/* Summary */}
              <button
                onClick={() =>
                  setExpandedSuggestion(expandedSuggestion === suggestion.name ? null : suggestion.name)
                }
                className="w-full text-left p-2 hover:bg-slate-600/50 transition"
              >
                <div className="flex items-start gap-2">
                  <div className="mt-0.5">{getCategoryIcon(suggestion.category)}</div>
                  <div className="flex-1 min-w-0">
                    <p className="font-mono text-sm text-white">{suggestion.name}</p>
                    <p className="text-xs text-gray-400">{suggestion.description}</p>
                    <div className="flex items-center gap-2 mt-1">
                      <span className="inline-block px-2 py-0.5 bg-slate-600 rounded text-xs text-gray-300">
                        {getCategoryLabel(suggestion.category)}
                      </span>
                      <span className="inline-block px-2 py-0.5 bg-blue-900 rounded text-xs text-blue-200">
                        {suggestion.confidence}% match
                      </span>
                      {suggestion.is_update && (
                        <span className="inline-block px-2 py-0.5 bg-green-900 rounded text-xs text-green-200">
                          Update available
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              </button>

              {/* Expanded Details */}
              {expandedSuggestion === suggestion.name && (
                <div className="px-2 pb-2 border-t border-slate-600 bg-slate-700/50 space-y-2">
                  <div className="bg-slate-800 rounded p-2">
                    <p className="text-xs text-gray-300 leading-relaxed">
                      <span className="font-semibold text-blue-300">Why this helps:</span> {suggestion.reasoning}
                    </p>
                  </div>

                  <div className="flex items-center justify-between text-xs">
                    <div>
                      {suggestion.installed_version ? (
                        <p className="text-gray-400">
                          Installed: <span className="text-yellow-300">{suggestion.installed_version}</span>
                          {suggestion.is_update && (
                            <>
                              <span className="text-gray-500"> → </span>
                              <span className="text-green-300">{suggestion.version}</span>
                            </>
                          )}
                        </p>
                      ) : (
                        <p className="text-gray-400">Not installed</p>
                      )}
                    </div>
                    <a
                      href={`https://pypi.org/project/${suggestion.name}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-blue-400 hover:text-blue-300"
                    >
                      PyPI ↗
                    </a>
                  </div>

                  <div className="flex gap-1">
                    <button
                      onClick={() => onInstall(suggestion.name, suggestion.version)}
                      className="flex-1 px-2 py-1 bg-blue-600 hover:bg-blue-700 rounded text-xs text-white font-medium transition flex items-center justify-center gap-1"
                    >
                      <Check size={12} />
                      Install {suggestion.version}
                    </button>
                    <button
                      onClick={() => handleIgnore(suggestion.name)}
                      className="flex-1 px-2 py-1 bg-slate-600 hover:bg-slate-500 rounded text-xs text-gray-300 font-medium transition flex items-center justify-center gap-1"
                    >
                      <X size={12} />
                      Ignore
                    </button>
                  </div>
                </div>
              )}
            </div>
          ))
        )}
      </div>

      {/* Footer */}
      <div className="border-t border-slate-700 p-2 bg-slate-700/30">
        <p className="text-xs text-gray-500 text-center">
          💡 Suggestions update as you code. Ignored libraries won't reappear.
        </p>
      </div>
    </div>
  )
}
