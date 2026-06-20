# Unified Search Architecture - Global Search Across Everything

**Vision:** One search box to find anything across the entire workspace
**Status:** Architecture Design
**Priority:** High (Core Feature)
**Implementation:** 4-6 weeks

---

## The Problem

Currently, users must:
- Search notebooks manually by opening each one
- Remember file paths to find files
- Search database content separately
- Look through execution history manually
- Recall variable names from memory
- Search chat history separately

**Solution:** Unified global search that covers everything

---

## Search Command Palette

**Trigger:** `Cmd/Ctrl+K` or Click search icon

```
┌─────────────────────────────────────────┐
│ Search everything...                 ✕  │
│                                         │
│ RECENT SEARCHES                         │
│ > import pandas                         │
│ > CREATE TABLE                          │
│ > def calculate_sum                     │
│                                         │
│ QUICK FILTERS                           │
│ [All] [Notebooks] [Code] [Files]       │
│ [Database] [Variables] [History]        │
│                                         │
│ RESULTS (showing 1-10 of 47)            │
│                                         │
│ 📓 analysis.ipynb                       │
│    Line 23: import pandas as pd        │
│    Line 24: df = pd.read_parquet()     │
│                                         │
│ 📄 utils.py                             │
│    Line 5: def calculate_sum(values):  │
│    Line 6:     return sum(values)      │
│                                         │
│ 🗄️ table_users (BigQuery)               │
│    Columns: user_id, email, created_at │
│                                         │
│ 💾 df (Variable)                        │
│    Type: DataFrame, Shape: (1000, 5)   │
│                                         │
│ ⏱️ Execution 2h ago                      │
│    Cell 5: SELECT COUNT(*) FROM users  │
│                                         │
│ 💬 AI Chat                              │
│    "How to optimize pandas queries?"   │
│                                         │
│ More results... (loading)               │
└─────────────────────────────────────────┘
```

---

## Search Categories

### 1. **Notebook Content Search**
```
Searches within notebook cells:
✓ Cell code (all languages)
✓ Cell output (text, tables, JSON)
✓ Cell markdown (comments, documentation)
✓ Cell metadata (tags, execution time)

Example: Search "def function_name"
Returns: All cells with that function definition
```

### 2. **File System Search**
```
Searches project files:
✓ File names
✓ File paths
✓ File content (code, text, JSON)
✓ File metadata (size, modified date)

Example: Search "requirements.txt"
Returns: File with location and preview
```

### 3. **Database/Table Search**
```
Searches connected data warehouses:
✓ Table names
✓ Column names
✓ Column descriptions
✓ Table schemas

Example: Search "users"
Returns: All tables named "users" in connected databases
```

### 4. **Variable/State Search**
```
Searches current session variables:
✓ Variable names
✓ Variable types
✓ Variable values (preview)
✓ Variable history

Example: Search "dataframe"
Returns: All DataFrame variables in workspace
```

### 5. **Execution History Search**
```
Searches past executions:
✓ Commands run
✓ Results/output
✓ Timestamps
✓ Success/error status

Example: Search "error"
Returns: All cells that errored with details
```

### 6. **Comment/Annotation Search**
```
Searches comments and annotations:
✓ Cell comments
✓ Inline notes
✓ Marked sections
✓ TODO items

Example: Search "@todo"
Returns: All cells with TODO markers
```

### 7. **AI Chat History Search**
```
Searches AI assistant conversations:
✓ Questions asked
✓ Responses received
✓ Context used
✓ Recommendations given

Example: Search "optimize performance"
Returns: All AI conversations about optimization
```

### 8. **Connection Search**
```
Searches external connections:
✓ Connection names
✓ Database/schema names
✓ Recent queries
✓ Connection status

Example: Search "snowflake"
Returns: All Snowflake connections and recent queries
```

---

## Smart Search Features

### Fuzzy Matching
```
Search: "readpq"
Matches: "read_parquet", "pd.read_parquet"

Search: "crtbl"
Matches: "CREATE TABLE"
```

### Type Ahead Suggestions
```
As user types "import":
✓ import numpy
✓ import pandas
✓ import matplotlib
✓ from collections import defaultdict
```

### Search Filters
```
Active filters appear below search:
[x] Notebooks [x] Files [ ] Database [ ] Variables

Show only: Notebooks AND Files
Hide: Database AND Variables
```

### Result Preview
```
Hover over result shows:
- Context (surrounding lines)
- Snippet of match
- File/cell location
- Relevance score
```

### Search History
```
Saved searches:
- Last 10 searches
- Starred searches
- Custom filters saved
- Search templates
```

---

## Advanced Search Syntax

### Boolean Operators
```
"function_name" AND "return"     → Both terms
"class" OR "def"                 → Either term
"try" NOT "except"               → Exclude term
```

### Scope Modifiers
```
notebook:analysis.ipynb          → In specific notebook
file:*.py                        → Specific file type
db:snowflake.public              → In specific schema
var:DataF*                       → Variables matching pattern
```

### Date Range
```
after:2026-06-01                 → After date
before:2026-06-20                → Before date
last:24h                         → Last 24 hours
last:week                        → Last 7 days
```

### Status Filters
```
status:error                     → Failed executions
status:success                   → Successful cells
modified:true                    → Unsaved changes
```

### Example Queries
```
notebook:analysis.ipynb AND "SELECT" AND status:error
file:*.py AND "def calculate" NOT "test"
var:DataFrame AND after:2026-06-01
db:snowflake AND "users" AND modified:true
```

---

## Search Implementation

### Backend (Rust)

```rust
pub struct SearchEngine {
    notebook_index: TantivyIndex,
    file_index: TantivyIndex,
    db_index: TantivyIndex,
    variable_store: Arc<VariableStore>,
    execution_history: ExecutionHistory,
}

impl SearchEngine {
    pub async fn search(&self, query: SearchQuery) -> Result<SearchResults> {
        let mut results = vec![];
        
        if query.scope.contains("notebooks") {
            results.extend(self.search_notebooks(&query.text).await?);
        }
        if query.scope.contains("files") {
            results.extend(self.search_files(&query.text).await?);
        }
        if query.scope.contains("database") {
            results.extend(self.search_databases(&query.text).await?);
        }
        if query.scope.contains("variables") {
            results.extend(self.search_variables(&query.text).await?);
        }
        if query.scope.contains("history") {
            results.extend(self.search_history(&query.text).await?);
        }
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(SearchResults {
            results: results.into_iter().take(100).collect(),
            total_count: results.len(),
            query_time_ms: elapsed,
        })
    }
    
    pub async fn search_notebooks(&self, query: &str) -> Result<Vec<SearchResult>> {
        let searcher = self.notebook_index.reader()?;
        let results = searcher.search(query)?;
        Ok(results.iter().map(|r| SearchResult {
            type_: "notebook",
            title: r.notebook_name,
            location: format!("{}:{}", r.notebook_id, r.cell_id),
            preview: r.content.lines().take(3).collect::<Vec<_>>().join(" "),
            score: r.relevance_score,
        }).collect())
    }
    
    pub async fn search_databases(&self, query: &str) -> Result<Vec<SearchResult>> {
        let mut results = vec![];
        for connection in self.connections.iter() {
            // Search table names, columns, schemas
            let tables = connection.search_tables(query).await?;
            results.extend(tables);
        }
        Ok(results)
    }
    
    pub async fn search_variables(&self, query: &str) -> Result<Vec<SearchResult>> {
        let variables = self.variable_store.list();
        Ok(variables
            .iter()
            .filter(|v| v.name.contains(query))
            .map(|v| SearchResult {
                type_: "variable",
                title: v.name.clone(),
                location: format!("session:{}", v.created_at),
                preview: format!("{}: {}", v.type_, v.value_preview),
                score: 1.0,
            })
            .collect())
    }
}

pub struct SearchQuery {
    pub text: String,
    pub scope: Vec<String>,  // ["notebooks", "files", "database", ...]
    pub filters: HashMap<String, String>,
    pub date_range: Option<DateRange>,
}

pub struct SearchResult {
    pub type_: String,  // "notebook", "file", "table", "variable", etc.
    pub title: String,
    pub location: String,
    pub preview: String,
    pub score: f32,
}

pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub total_count: usize,
    pub query_time_ms: u64,
}
```

### Frontend (React)

```tsx
// UnifiedSearch.tsx
import React, { useState, useEffect } from 'react'
import { Search, X } from 'lucide-react'

interface SearchResult {
  type: string
  title: string
  location: string
  preview: string
  score: number
}

export default function UnifiedSearch() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [isOpen, setIsOpen] = useState(false)
  const [filters, setFilters] = useState({
    notebooks: true,
    files: true,
    database: true,
    variables: true,
    history: true,
    comments: true,
    ai: true,
    connections: true,
  })
  const [activeResult, setActiveResult] = useState(0)

  useEffect(() => {
    // Register Cmd+K shortcut
    const handleKeyPress = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        setIsOpen(true)
      }
      if (e.key === 'Escape') {
        setIsOpen(false)
      }
      if (e.key === 'ArrowDown') {
        setActiveResult((prev) => Math.min(prev + 1, results.length - 1))
      }
      if (e.key === 'ArrowUp') {
        setActiveResult((prev) => Math.max(prev - 1, 0))
      }
      if (e.key === 'Enter' && results[activeResult]) {
        handleResultClick(results[activeResult])
      }
    }
    window.addEventListener('keydown', handleKeyPress)
    return () => window.removeEventListener('keydown', handleKeyPress)
  }, [results, activeResult])

  const performSearch = async (searchQuery: string) => {
    if (!searchQuery.trim()) {
      setResults([])
      return
    }

    const response = await fetch('/api/search', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        query: searchQuery,
        filters: Object.entries(filters)
          .filter(([_, v]) => v)
          .map(([k]) => k),
      }),
    })

    const data = await response.json()
    setResults(data.results)
    setActiveResult(0)
  }

  const handleResultClick = (result: SearchResult) => {
    switch (result.type) {
      case 'notebook':
        navigateToNotebook(result.location)
        break
      case 'file':
        openFile(result.location)
        break
      case 'variable':
        focusVariable(result.title)
        break
      case 'database':
        openDatabase(result.location)
        break
      case 'ai':
        scrollToAIChat(result.location)
        break
    }
    setIsOpen(false)
  }

  return (
    <>
      {/* Search Button in Header */}
      <button
        onClick={() => setIsOpen(true)}
        className="flex items-center gap-2 px-4 py-2 bg-gray-100 dark:bg-gray-800 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700"
      >
        <Search size={18} />
        <span className="text-sm text-gray-600 dark:text-gray-400">
          Cmd+K to search
        </span>
      </button>

      {/* Search Modal */}
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-start justify-center pt-20 bg-black/50">
          <div className="w-full max-w-2xl bg-white dark:bg-gray-900 rounded-lg shadow-lg">
            {/* Search Input */}
            <div className="p-4 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-center gap-3">
                <Search size={20} className="text-gray-400" />
                <input
                  autoFocus
                  type="text"
                  value={query}
                  onChange={(e) => {
                    setQuery(e.target.value)
                    performSearch(e.target.value)
                  }}
                  placeholder="Search notebooks, files, databases, variables..."
                  className="flex-1 bg-transparent text-lg focus:outline-none"
                />
                <button
                  onClick={() => setIsOpen(false)}
                  className="text-gray-400 hover:text-gray-600"
                >
                  <X size={20} />
                </button>
              </div>

              {/* Filter Chips */}
              <div className="mt-3 flex flex-wrap gap-2">
                {Object.entries(filters).map(([key, value]) => (
                  <button
                    key={key}
                    onClick={() =>
                      setFilters((prev) => ({
                        ...prev,
                        [key]: !prev[key as keyof typeof filters],
                      }))
                    }
                    className={`px-3 py-1 rounded-full text-sm transition ${
                      value
                        ? 'bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-100'
                        : 'bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400'
                    }`}
                  >
                    {key}
                  </button>
                ))}
              </div>
            </div>

            {/* Results */}
            <div className="max-h-96 overflow-y-auto">
              {results.length > 0 ? (
                results.map((result, idx) => (
                  <div
                    key={`${result.type}-${result.location}`}
                    onClick={() => handleResultClick(result)}
                    className={`p-4 border-b border-gray-100 dark:border-gray-800 cursor-pointer transition ${
                      idx === activeResult
                        ? 'bg-blue-50 dark:bg-blue-900/20'
                        : 'hover:bg-gray-50 dark:hover:bg-gray-800/50'
                    }`}
                  >
                    <div className="flex items-start gap-3">
                      <span className="text-lg">{getResultIcon(result.type)}</span>
                      <div className="flex-1 min-w-0">
                        <div className="font-medium">{result.title}</div>
                        <div className="text-sm text-gray-600 dark:text-gray-400">
                          {result.preview}
                        </div>
                        <div className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                          {result.location}
                        </div>
                      </div>
                      <div className="text-xs text-gray-400">
                        {(result.score * 100).toFixed(0)}%
                      </div>
                    </div>
                  </div>
                ))
              ) : query ? (
                <div className="p-8 text-center text-gray-500 dark:text-gray-400">
                  No results for "{query}"
                </div>
              ) : (
                <div className="p-8 text-center text-gray-500 dark:text-gray-400">
                  Start typing to search...
                </div>
              )}
            </div>

            {/* Help Text */}
            <div className="p-3 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-500 dark:text-gray-400">
              <div className="flex justify-between">
                <span>
                  <kbd>↑↓</kbd> Navigate | <kbd>Enter</kbd> Select |{' '}
                  <kbd>Esc</kbd> Close
                </span>
                <span>Advanced: AND, OR, NOT, date:, file:, db:</span>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  )
}

function getResultIcon(type: string) {
  const icons: Record<string, string> = {
    notebook: '📓',
    file: '📄',
    database: '🗄️',
    variable: '💾',
    history: '⏱️',
    comment: '💬',
    ai: '🤖',
    connection: '🔗',
  }
  return icons[type] || '🔍'
}
```

### API Endpoints

```
POST /api/search
  Body: {
    query: "search text",
    filters: ["notebooks", "files", "database", ...],
    date_range: { from: "2026-06-01", to: "2026-06-20" }
  }
  Returns: {
    results: [SearchResult...],
    total_count: 47,
    query_time_ms: 245
  }

GET /api/search/suggest?q=import
  Returns: Autocomplete suggestions

GET /api/search/history
  Returns: Previous searches

POST /api/search/save
  Body: { query: "...", name: "My saved search" }
  Saves search for later
```

---

## Search Indexing Strategy

### Real-time Updates
```
On every change:
- User edits cell → Update notebook index (100ms)
- File saved → Update file index (50ms)
- Variable created → Update variable store (10ms)
- Query executed → Update history (50ms)
```

### Background Indexing
```
Periodic full indexing:
- Every 5 minutes: Re-index all notebooks
- Every 10 minutes: Re-index all files
- Every 30 seconds: Update recent results cache
- On startup: Full index rebuild
```

### Performance Targets
```
Search latency:
- < 100ms for 1000 results
- < 500ms for full database scan
- Display first 10 results in < 50ms
```

---

## Search Keyboard Shortcuts

```
Cmd/Ctrl+K           Global search (focus)
Cmd/Ctrl+Shift+F     File search (full text)
Cmd/Ctrl+F           Find in current notebook
Cmd/Ctrl+H           Search history
Cmd/Ctrl+R           Recent searches
/                    Command search (in search)
?                    Show help/syntax
```

---

## Search UI Locations

### 1. **Header Search Bar** (Always Visible)
```
┌─────────────────────────────────────┐
│ 🔍 Search (Cmd+K) [Quick access]    │
└─────────────────────────────────────┘
```

### 2. **Command Palette**
```
Cmd+Shift+P → Type anything → Search results
```

### 3. **In-Editor Search**
```
Cmd+F → Search current notebook only
```

### 4. **Sidebar Search**
```
Each sidebar view has search:
- Notebook list search
- File tree search
- Database table search
- Variable list search
```

---

## Example Search Workflows

### 1. Find DataFrame Variables
```
Query: var:DataFrame
Filter: [x] Variables [ ] Others
Result: df, data, analysis_df, results
```

### 2. Find All Errors in Last 24h
```
Query: status:error last:24h
Filters: [x] History
Result: 3 failed cells with stack traces
```

### 3. Find SQL Queries on Production
```
Query: db:snowflake.prod AND "SELECT"
Filters: [x] Database
Result: All production queries in notebook
```

### 4. Find AI Conversations About Performance
```
Query: ai:"optimize" OR "performance"
Filters: [x] AI Chat
Result: Previous optimization discussions
```

### 5. Find TODOs
```
Query: "@todo" OR "TODO"
Filters: [x] Comments
Result: All cells with pending work
```

---

## Advanced Features

### Search Templates
```
"Find errors in last 24h"
  → Expands to: status:error last:24h

"Find all big queries"
  → Expands to: "SELECT" AND "1000000"

"Find my annotations"
  → Expands to: modified:true AND comment
```

### Smart Results Ranking
```
1. Exact match (100% relevance)
2. Fuzzy match in title (80-99%)
3. Fuzzy match in preview (50-79%)
4. Whole word match (40-49%)
5. Partial word match (1-39%)

Recent results ranked higher (×1.5 boost)
```

### Search Analytics
```
Track:
- Most searched terms
- Search success rate (user clicked result)
- Average search latency
- Popular filter combinations
```

---

## Benefits

1. **Unified Access** - One place to search everything
2. **Fast Navigation** - Jump anywhere in seconds
3. **Cross-Context** - Find code, data, history all at once
4. **Powerful Syntax** - Advanced searches for power users
5. **Intelligent** - Fuzzy matching, ranking, suggestions
6. **Keyboard-First** - No mouse needed

---

This unified search transforms PrismNote into a complete searchable workspace, making it easy for users to find anything they need instantly.
