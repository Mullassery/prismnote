# Cursor Integration Strategy - Enhanced AI Code Suggestions

**Vision:** Leverage Cursor's AI capabilities alongside PrismNote's notebook interface
**Status:** Integration Architecture
**Priority:** High (Enhanced Developer Experience)
**Implementation:** 3-4 weeks

---

## What is Cursor?

Cursor is an AI-first code editor with:
- **Superior code suggestions** (Claude-based)
- **Context-aware completions** (understands entire codebase)
- **Cmd+K command interface** (fast operations)
- **Codebase indexing** (semantic understanding)
- **Chat interface** (conversational coding)
- **Multi-file edits** (coordinated changes)

---

## Integration Approaches

### Approach 1: Cursor as External Editor (Easy)

**Allow users to:**
```
1. Export notebook to .py file
2. Open in Cursor for editing
3. Import changes back to notebook
4. Keep both in sync
```

**Flow:**
```
PrismNote Notebook
    ↓
[Export to Python]
    ↓
Cursor Editor ← User edits with AI help
    ↓
[Import Changes]
    ↓
PrismNote Notebook (Updated)
```

**Implementation:** 2-3 hours
- Add "Edit in Cursor" button
- Export cell code to .py
- Import diff/changes back
- Preserve cell structure

---

### Approach 2: Cursor API Integration (Medium)

**Embed Cursor's AI directly in PrismNote:**

```
In-App Features:
- Cmd+K for fast operations (just like Cursor)
- Chat interface (Cursor-style)
- Multi-file edits (across cells)
- Smart suggestions (Cursor's AI)
```

**Implementation:** 4-6 weeks
- Use Cursor's APIs if available
- Or implement similar features using Claude API
- Mirror Cursor's UX patterns
- Maintain consistency with Cursor experience

---

### Approach 3: Hybrid Workspace (Advanced)

**Best of both worlds:**
```
Option A: Side-by-side editing
- Cursor window on one screen
- PrismNote on another
- Real-time sync between them

Option B: Integrated editor
- Embed Cursor's editor in PrismNote
- Use Cursor's AI suggestions
- Run code in PrismNote's notebook kernel
```

---

## Recommended: Adopt Cursor's Best UX Patterns

Rather than hard integration, adopt Cursor's proven UX that makes it special:

### 1. Cmd+K Command Interface

**In PrismNote:**
```
Cmd+K opens command palette with AI suggestions:
- Generate unit tests
- Add docstrings
- Refactor this code
- Optimize performance
- Add error handling
- Write comments
```

**Implementation:**
```tsx
// CommandK.tsx
function CommandK() {
  return (
    <CommandPalette
      suggestions={[
        { icon: '✓', text: 'Add type hints' },
        { icon: '📝', text: 'Generate docstring' },
        { icon: '🔧', text: 'Refactor function' },
        { icon: '⚡', text: 'Optimize code' },
        { icon: '🛡️', text: 'Add error handling' },
        { icon: '📋', text: 'Add comments' },
        { icon: '✅', text: 'Generate tests' },
      ]}
    />
  )
}
```

### 2. Chat Interface (Like Cursor's Chat)

**In PrismNote:**
```
Cmd+L opens inline chat in cell:
User: "Explain this algorithm"
Claude: Detailed explanation with code blocks
User: "Make it more efficient"
Claude: Optimized version with benchmark

All within the cell context
```

### 3. Inline Suggestions

**As user types in editor:**
```python
def calculate_| ← Cursor shows: (suggestions)
  - calculate_sum()
  - calculate_average()
  - calculate_variance()
  - calculate_correlation()
```

**Works for:**
- Function names
- Variable names
- Import statements
- Code blocks
- Documentation

### 4. Context-Aware Code Actions

**Right-click on code:**
```
- Explain this code
- Add tests for this
- Optimize this function
- Add error handling
- Generate docstring
- Refactor to use [pattern]
- Compare with [alternative]
```

### 5. Multi-Cell Refactoring

**Like Cursor's multi-file edits:**
```
User: "Extract this logic into a utility function"
PrismNote AI:
1. Creates new cell with utility function
2. Updates all cells that use the logic
3. Adds appropriate imports
4. Shows diff for approval
```

---

## Implementation: Cursor-Inspired Features

### Component 1: CommandK with AI

```tsx
// CursorCommandK.tsx
import React, { useState, useEffect } from 'react'
import { MessageCircle } from 'lucide-react'

export default function CursorCommandK({ selectedCode }: { selectedCode: string }) {
  const [open, setOpen] = useState(false)
  const [input, setInput] = useState('')
  const [suggestions, setSuggestions] = useState<string[]>([])

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        setOpen(true)
      }
      if ((e.metaKey || e.ctrlKey) && e.key === 'l') {
        e.preventDefault()
        // Open inline chat
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [])

  const handleCommand = async (command: string) => {
    const response = await fetch('/api/ai/execute-command', {
      method: 'POST',
      body: JSON.stringify({
        command,
        code: selectedCode,
        context: getNotebookContext(),
      }),
    })
    const result = await response.json()
    applyResult(result)
  }

  const cursorCommands = [
    { icon: '✓', text: 'Add type hints', cmd: 'add_type_hints' },
    { icon: '📝', text: 'Generate docstring', cmd: 'gen_docstring' },
    { icon: '🔧', text: 'Refactor code', cmd: 'refactor' },
    { icon: '⚡', text: 'Optimize code', cmd: 'optimize' },
    { icon: '🛡️', text: 'Add error handling', cmd: 'add_errors' },
    { icon: '📋', text: 'Add comments', cmd: 'add_comments' },
    { icon: '✅', text: 'Generate tests', cmd: 'gen_tests' },
    { icon: '🔍', text: 'Explain code', cmd: 'explain' },
  ]

  return (
    <>
      {open && (
        <div className="fixed inset-0 z-50 flex items-start justify-center pt-20">
          <div className="w-full max-w-2xl bg-white dark:bg-gray-900 rounded-lg shadow-xl">
            <div className="p-4 border-b">
              <input
                autoFocus
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder="Ask AI to modify your code... (Cmd+K)"
                className="w-full bg-transparent text-lg focus:outline-none"
              />
            </div>

            <div className="p-4 grid grid-cols-2 gap-2 max-h-96 overflow-y-auto">
              {cursorCommands.map((cmd) => (
                <button
                  key={cmd.cmd}
                  onClick={() => handleCommand(cmd.cmd)}
                  className="p-3 text-left hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition"
                >
                  <div className="text-lg mb-1">{cmd.icon}</div>
                  <div className="font-medium text-sm">{cmd.text}</div>
                </button>
              ))}
            </div>

            <div className="p-3 border-t text-xs text-gray-500">
              <kbd>Cmd+K</kbd> for AI commands | <kbd>Cmd+L</kbd> for inline chat |{' '}
              <kbd>Esc</kbd> to close
            </div>
          </div>
        </div>
      )}
    </>
  )
}
```

### Component 2: Inline Chat (Cursor-Style)

```tsx
// InlineChat.tsx
import React, { useState } from 'react'

export default function InlineChat({ cellId }: { cellId: string }) {
  const [chatOpen, setChatOpen] = useState(false)
  const [messages, setMessages] = useState<Array<{ role: string; content: string }>>([])
  const [input, setInput] = useState('')

  const handleChat = async (message: string) => {
    const response = await fetch('/api/ai/cell-chat', {
      method: 'POST',
      body: JSON.stringify({
        cellId,
        message,
        context: getCellContext(cellId),
        conversationHistory: messages,
      }),
    })
    const result = await response.json()
    setMessages([...messages, { role: 'user', content: message }, { role: 'assistant', content: result.response }])
  }

  return (
    <div className="cell-chat">
      {chatOpen && (
        <div className="chat-panel border-l-2 border-blue-500 pl-4 py-2">
          <div className="space-y-2 max-h-48 overflow-y-auto">
            {messages.map((msg, idx) => (
              <div key={idx} className={`text-sm ${msg.role === 'user' ? 'text-blue-600' : 'text-gray-600'}`}>
                <strong>{msg.role === 'user' ? 'You' : 'Claude'}:</strong> {msg.content}
              </div>
            ))}
          </div>
          <div className="mt-2 flex gap-2">
            <input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyPress={(e) => {
                if (e.key === 'Enter') {
                  handleChat(input)
                  setInput('')
                }
              }}
              placeholder="Ask about this cell... (Cmd+L)"
              className="flex-1 px-2 py-1 text-sm border rounded"
            />
            <button onClick={() => handleChat(input)} className="px-2 py-1 bg-blue-500 text-white rounded">
              Send
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
```

### Component 3: Smart Completions

```tsx
// SmartAutocomplete.tsx
interface Suggestion {
  text: string
  description: string
  kind: 'function' | 'variable' | 'import' | 'snippet'
}

async function getSmartSuggestions(
  code: string,
  position: number,
  context: NotebookContext
): Promise<Suggestion[]> {
  const response = await fetch('/api/ai/completions', {
    method: 'POST',
    body: JSON.stringify({
      code,
      position,
      notebookContext: context,
      recentCells: getRecentCells(),
    }),
  })

  const suggestions = await response.json()
  return suggestions.map((s: any) => ({
    text: s.completion,
    description: s.description,
    kind: s.type,
  }))
}
```

---

## Cursor-Like Features to Implement

### 1. Smart Code Actions (Ctrl+.)
```
Right-click on code:
✓ Add type hints
✓ Generate docstring
✓ Add error handling
✓ Extract function
✓ Inline variable
✓ Rename symbol
✓ Generate tests
✓ Optimize code
```

### 2. Code Lens (Inline Info)
```
Above each function:
"Claude can explain this → Click to see"
"Test coverage: 0% → Generate tests"
"Performance: Consider optimizing → See suggestions"
```

### 3. Inline Diff Preview
```
Before applying AI suggestion:
Code action → Show diff
User reviews → Accept/Reject/Edit
Smooth apply animation
```

### 4. Multi-Cell Operations
```
Select multiple cells:
"Refactor these cells into utility"
"Extract common logic"
"Merge cells with similar functionality"
→ AI proposes coordinated changes
```

### 5. AI-Powered Search
```
Search with natural language:
"Find all database queries"
"Show me error handling code"
"Where do we process user input?"
→ Uses AI to understand intent, not just regex
```

---

## Keyboard Shortcuts (Cursor-Inspired)

```
Cmd+K              Execute AI command
Cmd+L              Inline chat in cell
Ctrl+.             Quick fixes & code actions
Cmd+/              Comment/uncomment
Cmd+D              Select word/next occurrence
Cmd+Shift+L        Multi-cursor editing
Cmd+I              Inline refactoring
Cmd+Shift+J        Generate tests
Cmd+Shift+D        Generate docstring
```

---

## Backend Implementation

### API Endpoints

```
POST /api/ai/execute-command
  Body: { command: "add_type_hints", code: "...", context: {...} }
  Returns: Modified code with changes

POST /api/ai/cell-chat
  Body: { cellId: "...", message: "...", history: [...] }
  Returns: { response: "...", suggestions: [...] }

POST /api/ai/completions
  Body: { code: "...", position: 123, context: {...} }
  Returns: [{ completion: "...", description: "..." }]

POST /api/ai/code-actions
  Body: { code: "...", range: {...}, context: {...} }
  Returns: [{ title: "...", command: "..." }]

POST /api/ai/multi-cell-refactor
  Body: { cellIds: [...], operation: "extract_function" }
  Returns: { cellUpdates: {...}, newCell: {...} }
```

### Command Implementations

```rust
pub enum AICommand {
    AddTypeHints,
    GenerateDocstring,
    RefactorCode,
    OptimizeCode,
    AddErrorHandling,
    AddComments,
    GenerateTests,
    ExplainCode,
    ExtractFunction,
    InlineVariable,
}

pub async fn execute_command(
    command: AICommand,
    code: &str,
    context: NotebookContext,
) -> Result<String> {
    match command {
        AICommand::AddTypeHints => add_type_hints(code, &context).await,
        AICommand::GenerateDocstring => gen_docstring(code, &context).await,
        AICommand::RefactorCode => refactor_code(code, &context).await,
        // ... etc
    }
}
```

---

## Integration with Claude API

```rust
pub async fn call_claude_for_ai_command(
    command: &str,
    code: &str,
    context: &NotebookContext,
) -> Result<String> {
    let prompt = match command {
        "add_type_hints" => format!(
            "Add Python type hints to this code:\n\n{}\n\nReturn only the modified code.",
            code
        ),
        "gen_docstring" => format!(
            "Add a comprehensive docstring to this code:\n\n{}\n\nReturn the code with added docstring.",
            code
        ),
        // ... etc
    };

    let response = client.messages.create(
        claude_sdk::CreateMessageRequest {
            model: "claude-opus-4-1".to_string(),
            max_tokens: 2048,
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            system: Some(vec![
                ContentBlock::Text(TextBlock {
                    text: format!("You are an expert Python developer. Available context:\n{:?}", context),
                    _type: "text".to_string(),
                })
            ]),
        }
    ).await?;

    Ok(response.content[0].text.clone())
}
```

---

## User Experience Flow

### Example 1: Add Type Hints (Cursor-style)
```
1. User selects function code
2. User presses Cmd+K
3. Clicks "Add type hints"
4. AI shows:
   Before: def calculate(x, y):
   After:  def calculate(x: float, y: float) -> float:
5. User reviews diff
6. Clicks "Apply"
7. Code updated in cell
```

### Example 2: Inline Chat (Cursor-style)
```
1. User presses Cmd+L in cell
2. Chat opens to right of code
3. User types: "Make this more efficient"
4. Claude responds with optimized version
5. User can ask follow-up questions
6. Chat context includes cell code and dependencies
```

### Example 3: Multi-Cell Refactor (Novel)
```
1. User selects 3 cells
2. Presses Cmd+K
3. Selects "Extract common logic"
4. AI:
   - Creates new utility cell
   - Updates 3 cells to use utility
   - Shows coordinated diff
5. User reviews and applies
```

---

## Why This Matters

1. **Matches User Expectations** - Cursor users expect Cmd+K
2. **Faster Development** - AI-powered suggestions in every step
3. **Learning Tool** - Users learn best practices from AI
4. **Professional Feel** - Cursor-like UX feels premium
5. **Productivity Boost** - Context-aware help at point of need

---

## Comparison: Cursor vs PrismNote

| Feature | Cursor | PrismNote (Proposed) |
|---------|--------|---------------------|
| Cmd+K Commands | ✓ | ✓ |
| Inline Chat | ✓ | ✓ |
| Code Actions | ✓ | ✓ |
| Multi-file edits | ✓ | ✓ (multi-cell) |
| Notebook interface | ✗ | ✓ |
| SQL execution | ✗ | ✓ |
| Data visualization | ✗ | ✓ |
| Real-time collab | ✗ | ✓ (v0.4) |
| External connections | ✗ | ✓ |

---

## Implementation Timeline

### Week 1: Cmd+K Commands
- Command palette with AI actions
- Add type hints command
- Generate docstring command
- Test and refine

### Week 2: Inline Chat
- Chat component in cell editor
- Conversation memory
- Code context in prompts
- Follow-up questions

### Week 3: Code Actions
- Right-click context menu
- Quick fixes (Ctrl+.)
- Code lens above functions
- Refactor suggestions

### Week 4: Polish & Testing
- Keyboard shortcuts optimization
- Performance tuning
- User testing
- Documentation

---

This Cursor-inspired approach gives PrismNote's users the familiar, powerful AI experience they expect from Cursor, while adding notebook-specific advantages that Cursor doesn't have.
