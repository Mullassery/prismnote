# Integrated Tools Architecture - Terminal, Files, and AI

**Vision:** Seamless integration of terminal, file system, and AI assistance
**Status:** Architecture Design
**Implementation:** 6-8 weeks

---

## Core Philosophy

PrismNote should be a **complete data science workspace**, not just a notebook editor. This means integrating:
1. **Terminal/Shell** - Execute bash/shell commands
2. **File System Explorer** - Browse and manage files
3. **AI Assistant** - Real-time code help, explanations, debugging
4. All three visible and working together simultaneously

---

## Integrated Layout

```
┌──────────────────────────────────────────────────────────┐
│ Command Palette | Notebook Name | Settings | Help        │
├─┬────────────────────────┬──────────────────────────┬─────┤
│▪│                        │     CENTRAL EDITOR      │ AI  │
│▪│  Activity Bar           │   (Code Cells)          │     │
│▪│  (Notebooks)           │                        │ Chat│
│ │  (Files)               │ [In 1]: import pandas  │     │
│ │  (Search)              │ df = pd.read_parquet() │ Side│
│ │  (Git)                 │                        │ Bar │
│ │  (Terminal)            │ [Out 1]: DataFrame...  │     │
│ │  (Connections)         │                        │     │
│ │                        │ [In 2]: df.head()      │     │
│▪│  SIDEBAR               │                        │     │
│▪│  (240px,               │ [Out 2]: ...          │     │
│▪│   collapsible)         │                        │     │
│▪│                        │                        │     │
│▪│  File Structure:       │                        │     │
│▪│  ├── notebooks/        │                        │ ┌──┐│
│▪│  ├── data/             │                        │ │ ││
│▪│  ├── src/              │ Variables              │ │AI││
│▪│  │   ├── utils.py      │ x: int = 42           │ │  ││
│▪│  │   └── config.py     │ df: DataFrame         │ │ ││
│▪│  └── README.md         │ result: list          │ └──┘│
│▪│                        │                        │     │
├─┼────────────────────────┼──────────────────────────┼─────┤
│▪│  ╶╶╶╶╶╶╶╶╶╶╶╶╶╶╶      │                        │     │
│▪│  TERMINAL (200px)      │        INSPECTOR       │     │
│▪│  $ cd notebooks        │     (collapsible)      │     │
│▪│  $ ls -la              │                        │     │
│▪│  $ python script.py    │                        │     │
│▪│  Output...             │                        │     │
│▪│                        │                        │     │
└─┴────────────────────────┴──────────────────────────┴─────┘
```

---

## Three Integrated Panels

### 1. Left Sidebar - File & Project Explorer

**Contents:**
```
Project Structure
├── notebooks/
│   ├── analysis_001.ipynb
│   ├── ml_pipeline.ipynb
│   └── sql_queries.ipynb
├── data/
│   ├── raw/
│   │   └── dataset.csv
│   └── processed/
│       └── cleaned.parquet
├── src/
│   ├── __init__.py
│   ├── utils.py
│   ├── models.py
│   └── config.py
├── tests/
│   ├── test_utils.py
│   └── test_models.py
├── README.md
├── requirements.txt
└── .gitignore
```

**Features:**
- Right-click context menu (create, delete, rename, open)
- Drag-and-drop file operations
- File icons by type
- Git status indicators (modified, untracked)
- Search files (Cmd+P)
- Quick open (type filename)
- Breadcrumb navigation

---

### 2. Bottom Panel - Integrated Terminal

**Capabilities:**
```
╔═ Terminal (bash/zsh/powershell) ════════════════════╗
║ $ pwd                                               ║
║ /Users/user/prismnote/notebooks                    ║
║                                                     ║
║ $ python script.py                                  ║
║ Processing data...                                  ║
║ Done! Results saved.                                ║
║                                                     ║
║ $ git status                                        ║
║ On branch main                                      ║
║ Changes not staged for commit:                      ║
║   modified: analysis.ipynb                          ║
║                                                     ║
║ $ _                                                 ║
╚═════════════════════════════════════════════════════╝
```

**Features:**
- Full bash/shell access
- File navigation
- Package management (pip, npm, etc.)
- Git operations
- System commands
- Output capture and display
- Copy/paste support
- Clear command

**Terminal Types Available:**
1. **Bash** - Default shell
2. **Python REPL** - Direct Python execution
3. **SQL Shell** - Database queries (DuckDB, Snowflake, etc.)
4. **Git Shell** - Version control commands
5. **System Shell** - General commands

---

### 3. Right Panel - AI Assistant

**Integrated AI Features:**

```
╔═ AI Assistant ══════════════════════════════════════╗
║ 🤖 Claude - Code Assistant                          ║
├─────────────────────────────────────────────────────┤
║                                                     ║
║ User: Explain this pandas code                     ║
║                                                     ║
║ df.groupby('category')\.agg({                      ║
║   'value': 'sum',                                   ║
║   'count': 'size'                                   ║
║ })                                                  ║
║                                                     ║
║ Claude: This code groups data by 'category' and   ║
║ calculates the sum of 'value' and the number of    ║
║ items in each group...                             ║
║                                                     ║
║ [Helpful Buttons]                                   ║
║ [Add to cell] [More detail] [Copy] [Try it]       ║
║                                                     ║
│ [Your question............................... ⏎]  │
╚═════════════════════════════════════════════════════╝
```

**AI Capabilities:**
- **Code Explanation** - Explain selected code
- **Code Generation** - Generate code from description
- **Debugging** - Analyze errors and suggest fixes
- **Optimization** - Suggest performance improvements
- **Documentation** - Generate docstrings
- **Refactoring** - Suggest better structure
- **Testing** - Generate test cases
- **Queries** - Build SQL queries

**AI Integration Points:**
1. Right-click cell → "Explain with AI"
2. Error occurs → Automatic AI suggestion
3. Question input → Real-time AI response
4. Code selection → Context-aware suggestions

---

## Layout CSS

```css
.integrated-workspace {
  display: grid;
  grid-template-columns: 50px 240px 1fr 350px;
  grid-template-rows: 40px 1fr 200px;
  gap: 0;
  height: 100vh;
  
  grid-template-areas:
    "header header header header"
    "activity sidebar editor ai"
    "terminal terminal terminal ai";
}

.activity-bar {
  grid-area: activity;
  width: 50px;
  
  /* Icons: Notebooks, Files, Search, Git, Terminal, Connections, Settings */
}

.file-explorer {
  grid-area: sidebar;
  width: 240px;
  border-right: 1px solid var(--border-light);
  overflow-y: auto;
}

.editor-panel {
  grid-area: editor;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  /* Main notebook cells */
}

.terminal-panel {
  grid-area: terminal;
  height: 200px;
  border-top: 1px solid var(--border-light);
  background: #1e1e1e; /* Dark terminal background */
  overflow-y: auto;
  font-family: 'Monaco', 'Menlo', monospace;
}

.ai-assistant {
  grid-area: ai;
  width: 350px;
  border-left: 1px solid var(--border-light);
  overflow-y: auto;
  background: var(--bg-secondary);
  padding: 16px;
  
  display: flex;
  flex-direction: column;
  
  > .messages {
    flex: 1;
    overflow-y: auto;
    margin-bottom: 16px;
  }
  
  > .input {
    display: flex;
    gap: 8px;
    
    input {
      flex: 1;
    }
    
    button {
      width: 40px;
    }
  }
}
```

---

## Terminal Implementation

### Features

1. **Xterm.js Integration**
   - Full terminal emulation
   - ANSI color support
   - Copy/paste
   - Resize handling

2. **Backend (Rust)**
   ```rust
   pub struct PtyManager {
       processes: HashMap<String, PtyProcess>,
   }
   
   impl PtyManager {
       pub async fn execute(&self, cmd: String) -> Result<Output>
       pub async fn stream(&self, cmd: String) -> Result<Stream>
       pub async fn interrupt(&self, pid: u32) -> Result<()>
   }
   ```

3. **Multi-Shell Support**
   - Bash/Zsh (default)
   - Python REPL
   - SQL interpreter
   - Git shell
   - Custom interpreters

4. **Command Tracking**
   - History (searchable)
   - Favorites
   - Repeat command
   - Clear output

---

## File Explorer Features

### Implementation

```tsx
// FileExplorer.tsx
function FileExplorer() {
  const [files, setFiles] = useState<FileTree>()
  
  const handleCreate = (path: string, type: 'file' | 'folder') => {
    // Create file/folder
  }
  
  const handleDelete = (path: string) => {
    // Delete with confirmation
  }
  
  const handleRename = (path: string, newName: string) => {
    // Rename file/folder
  }
  
  const handleDragDrop = (source: string, target: string) => {
    // Move files via drag-drop
  }
  
  const handleDoubleClick = (path: string) => {
    // Open file in editor/preview
  }
  
  return (
    <div className="file-explorer">
      <div className="toolbar">
        <input placeholder="Search files..." />
        <button onClick={() => createFile()}>+</button>
      </div>
      <Tree items={files} />
    </div>
  )
}
```

### Features
- Recursive folder display
- File type icons
- Git status indicators
- Right-click menu
- Drag-drop support
- Quick open (Cmd+P)
- File search
- Create/delete/rename

---

## AI Assistant Implementation

```tsx
// AIAssistant.tsx
function AIAssistant() {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  
  const handleContextRequest = (context: 'cell' | 'error' | 'selection') => {
    // Prepare context and send to AI
    const prompt = prepareContext(context, selectedCode)
    sendToAI(prompt)
  }
  
  const handleUserMessage = async (text: string) => {
    const response = await fetch('/api/ai/chat', {
      method: 'POST',
      body: JSON.stringify({
        message: text,
        context: {
          selectedCode: getSelection(),
          recentErrors: getRecentErrors(),
          variables: getVariables(),
          notebook: getCurrentNotebook(),
        }
      })
    })
    
    const result = await response.json()
    setMessages([...messages, result])
  }
  
  return (
    <div className="ai-assistant">
      <div className="messages">
        {messages.map(msg => (
          <Message key={msg.id} message={msg} />
        ))}
      </div>
      
      <div className="input-area">
        <input
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyPress={e => e.key === 'Enter' && handleUserMessage(input)}
          placeholder="Ask Claude anything..."
        />
        <button onClick={() => handleUserMessage(input)}>
          Send
        </button>
      </div>
      
      <QuickActions>
        <button onClick={() => handleContextRequest('selection')}>
          Explain selection
        </button>
        <button onClick={() => handleContextRequest('error')}>
          Fix error
        </button>
        <button onClick={() => handleContextRequest('cell')}>
          Optimize cell
        </button>
      </QuickActions>
    </div>
  )
}
```

---

## Keyboard Shortcuts

```
Terminal:
  Cmd/Ctrl+`         Toggle terminal
  Cmd/Ctrl+J         Focus terminal
  Ctrl+C             Interrupt process
  Ctrl+D             Exit terminal
  Cmd/Ctrl+L         Clear terminal

Files:
  Cmd/Ctrl+P         Quick open
  Cmd/Ctrl+Shift+P   Command palette
  Cmd/Ctrl+Shift+E   Focus explorer
  F2                 Rename
  Delete             Delete file
  Cmd/Ctrl+N         New file
  Cmd/Ctrl+Shift+N   New folder

AI:
  Cmd/Ctrl+Shift+I   Focus AI assistant
  Cmd/Ctrl+/         Ask about selection
  Cmd/Shift+?        AI help
```

---

## Implementation Roadmap

### Week 1-2: Terminal Integration
- Xterm.js setup
- Pty manager
- Command execution
- Output streaming

### Week 3-4: File Explorer
- Tree component
- File operations (create, delete, rename)
- Right-click menu
- Drag-drop support

### Week 5-6: AI Integration
- Chat UI
- Context preparation
- API integration
- Quick actions

### Week 7-8: Polish & Testing
- Cross-platform testing
- Keyboard shortcuts
- Responsive design
- Performance optimization

---

## Why This Matters

1. **Complete Workflow**
   - Write code in notebook
   - Execute in terminal
   - Manage files
   - Get instant AI help

2. **Professional Environment**
   - Familiar to developers (like VS Code)
   - All tools integrated
   - Efficient workflow

3. **Learning Aid**
   - AI explains code
   - Suggests improvements
   - Helps debug
   - Generates tests

4. **Data Science Focus**
   - SQL terminal for queries
   - File browser for data files
   - Python REPL for exploration
   - AI for complex tasks

---

This integrated architecture transforms PrismNote from a notebook editor into a complete data science IDE.
