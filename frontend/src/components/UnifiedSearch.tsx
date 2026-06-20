import { useState, useEffect, useRef, useCallback } from 'react'
import { Search, X, FileText, Database, Variable, History, MessageSquare, Link as LinkIcon } from 'lucide-react'

interface SearchResult {
  id: string
  title: string
  category: 'notebook' | 'file' | 'table' | 'variable' | 'history' | 'comment' | 'chat' | 'connection'
  content: string
  context?: string
  path?: string
  timestamp?: string
  score: number
}

interface SearchFilters {
  notebook?: boolean
  file?: boolean
  table?: boolean
  variable?: boolean
  history?: boolean
  comment?: boolean
  chat?: boolean
  connection?: boolean
}

export default function UnifiedSearch() {
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [loading, setLoading] = useState(false)
  const [filters, setFilters] = useState<SearchFilters>({
    notebook: true,
    file: true,
    table: true,
    variable: true,
    history: true,
    comment: true,
    chat: true,
    connection: true,
  })
  const inputRef = useRef<HTMLInputElement>(null)
  const resultsRef = useRef<HTMLDivElement>(null)

  // Keyboard shortcut: Cmd+K to open search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        setOpen(true)
        setTimeout(() => inputRef.current?.focus(), 100)
      }
      if (e.key === 'Escape' && open) {
        setOpen(false)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [open])

  // Fetch search results
  const performSearch = useCallback(async (searchQuery: string) => {
    if (!searchQuery.trim()) {
      setResults([])
      return
    }

    setLoading(true)
    try {
      const response = await fetch('/api/search', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query: searchQuery,
          filters: Object.entries(filters)
            .filter(([_, enabled]) => enabled)
            .map(([category]) => category),
        }),
      })

      if (response.ok) {
        const data = await response.json()
        setResults(data.results || [])
        setSelectedIndex(0)
      }
    } catch (error) {
      console.error('Search failed:', error)
    } finally {
      setLoading(false)
    }
  }, [filters])

  // Debounce search
  useEffect(() => {
    const timer = setTimeout(() => {
      performSearch(query)
    }, 100)
    return () => clearTimeout(timer)
  }, [query, performSearch])

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      setSelectedIndex((prev) => (prev + 1) % (results.length || 1))
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      setSelectedIndex((prev) => (prev - 1 + (results.length || 1)) % (results.length || 1))
    } else if (e.key === 'Enter' && results.length > 0) {
      e.preventDefault()
      handleSelectResult(results[selectedIndex])
    }
  }

  // Handle result selection
  const handleSelectResult = (result: SearchResult) => {
    // This would navigate to the result or perform an action
    console.log('Selected result:', result)
    setOpen(false)
    // TODO: Implement navigation based on result type
  }

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'notebook':
        return <FileText className="w-4 h-4" />
      case 'file':
        return <FileText className="w-4 h-4" />
      case 'table':
        return <Database className="w-4 h-4" />
      case 'variable':
        return <Variable className="w-4 h-4" />
      case 'history':
        return <History className="w-4 h-4" />
      case 'comment':
        return <MessageSquare className="w-4 h-4" />
      case 'chat':
        return <MessageSquare className="w-4 h-4" />
      case 'connection':
        return <LinkIcon className="w-4 h-4" />
      default:
        return <Search className="w-4 h-4" />
    }
  }

  const getCategoryLabel = (category: string) => {
    switch (category) {
      case 'notebook':
        return 'Notebook'
      case 'file':
        return 'File'
      case 'table':
        return 'Table'
      case 'variable':
        return 'Variable'
      case 'history':
        return 'History'
      case 'comment':
        return 'Comment'
      case 'chat':
        return 'Chat'
      case 'connection':
        return 'Connection'
      default:
        return 'Result'
    }
  }

  const toggleFilter = (category: keyof SearchFilters) => {
    setFilters((prev) => ({
      ...prev,
      [category]: !prev[category],
    }))
  }

  if (!open) return null

  return (
    <>
      {/* Modal backdrop */}
      <div
        className="fixed inset-0 z-40 bg-black/50 dark:bg-black/70"
        onClick={() => setOpen(false)}
      />

      {/* Search dialog */}
      <div className="fixed inset-0 z-50 flex items-start justify-center pt-20">
        <div className="w-full max-w-2xl bg-white dark:bg-gray-900 rounded-lg shadow-xl overflow-hidden">
          {/* Search input */}
          <div className="flex items-center gap-3 px-4 py-3 border-b border-gray-200 dark:border-gray-700">
            <Search className="w-5 h-5 text-gray-400 flex-shrink-0" />
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Search notebooks, files, tables, variables, history... (Cmd+K)"
              className="flex-1 bg-transparent text-lg focus:outline-none placeholder-gray-400 dark:placeholder-gray-500"
            />
            {query && (
              <button
                onClick={() => setQuery('')}
                className="p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded"
              >
                <X className="w-5 h-5" />
              </button>
            )}
          </div>

          {/* Filter pills */}
          <div className="flex flex-wrap gap-2 px-4 py-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
            {(
              [
                'notebook',
                'file',
                'table',
                'variable',
                'history',
                'comment',
                'chat',
                'connection',
              ] as Array<keyof SearchFilters>
            ).map((category) => (
              <button
                key={category}
                onClick={() => toggleFilter(category)}
                className={`px-3 py-1 rounded-full text-sm font-medium transition ${
                  filters[category]
                    ? 'bg-blue-500 text-white'
                    : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
                }`}
              >
                {getCategoryLabel(category)}
              </button>
            ))}
          </div>

          {/* Results */}
          <div
            ref={resultsRef}
            className="max-h-96 overflow-y-auto bg-white dark:bg-gray-900"
          >
            {loading && (
              <div className="flex items-center justify-center py-8">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
              </div>
            )}

            {!loading && results.length === 0 && query && (
              <div className="p-8 text-center text-gray-500 dark:text-gray-400">
                <p>No results found for "{query}"</p>
                <p className="text-sm mt-2">Try different keywords or check your filters</p>
              </div>
            )}

            {!loading && results.length === 0 && !query && (
              <div className="p-8 text-center text-gray-500 dark:text-gray-400">
                <p>Start typing to search</p>
                <p className="text-sm mt-2">Search across notebooks, files, tables, variables, history, and more</p>
              </div>
            )}

            {results.map((result, index) => (
              <button
                key={result.id}
                onClick={() => handleSelectResult(result)}
                className={`w-full px-4 py-3 text-left border-b border-gray-200 dark:border-gray-700 transition ${
                  index === selectedIndex
                    ? 'bg-blue-50 dark:bg-blue-900/30'
                    : 'hover:bg-gray-50 dark:hover:bg-gray-800/50'
                }`}
              >
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 mt-1 text-gray-400">
                    {getCategoryIcon(result.category)}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <h3 className="font-medium text-gray-900 dark:text-white truncate">
                        {result.title}
                      </h3>
                      <span className="text-xs px-2 py-1 rounded-full bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300 whitespace-nowrap">
                        {getCategoryLabel(result.category)}
                      </span>
                    </div>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mt-1 line-clamp-2">
                      {result.context || result.content}
                    </p>
                    {result.path && (
                      <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                        {result.path}
                      </p>
                    )}
                  </div>
                  {result.score && (
                    <div className="flex-shrink-0 text-xs text-gray-400">
                      {Math.round(result.score * 100)}%
                    </div>
                  )}
                </div>
              </button>
            ))}
          </div>

          {/* Footer */}
          <div className="px-4 py-2 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-xs text-gray-600 dark:text-gray-400">
            <kbd className="px-2 py-1 rounded bg-white dark:bg-gray-700">↑↓</kbd>
            <span className="mx-2">to navigate</span>
            <kbd className="px-2 py-1 rounded bg-white dark:bg-gray-700">Enter</kbd>
            <span className="mx-2">to select</span>
            <kbd className="px-2 py-1 rounded bg-white dark:bg-gray-700">Esc</kbd>
            <span className="mx-2">to close</span>
          </div>
        </div>
      </div>
    </>
  )
}
