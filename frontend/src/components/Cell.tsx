import Editor, { DiffEditor } from '@monaco-editor/react'
import MDPreview from '@uiw/react-markdown-preview'
import { useEffect, useRef, useState } from 'react'
import { Play, Trash2, Sparkles, Wand2, Check, X, Loader2, Square } from 'lucide-react'
import Output from './Output'
import { useNotebookStore } from '../hooks/useNotebook'
import { aiEdit, aiFix, aiExplain } from '../api/ai'
import { interruptKernel } from '../api/kernel'
import { subscribeCellStream } from '../api/stream'
import { registerOllamaCompletions } from '../api/autocomplete'
import { parseTraceback } from '../lib/pyerror'

interface CellProps {
  cell: any
  cellIndex: number
}

const isDark = () => document.documentElement.classList.contains('dark')

/** Pull a human-readable error string out of a cell's outputs, if any. */
function errorFromOutputs(outputs: any[]): string | null {
  for (const o of outputs ?? []) {
    if (o?.output_type === 'error') {
      if (Array.isArray(o.traceback) && o.traceback.length) return o.traceback.join('\n')
      if (o.ename || o.evalue) return `${o.ename ?? ''}: ${o.evalue ?? ''}`.trim()
      if (Array.isArray(o.text)) return o.text.join('')
      if (typeof o.text === 'string') return o.text
    }
  }
  return null
}

export default function Cell({ cell, cellIndex }: CellProps) {
  const [isEditing, setIsEditing] = useState(!cell.source.length)
  const { updateCell, deleteCell, executeCell, currentNotebook } = useNotebookStore()
  const [isExecuting, setIsExecuting] = useState(false)
  const [liveOut, setLiveOut] = useState('')

  // AI state
  const [aiOpen, setAiOpen] = useState(false)
  const [aiPrompt, setAiPrompt] = useState('')
  const [aiBusy, setAiBusy] = useState(false)
  const [aiError, setAiError] = useState<string | null>(null)
  const [proposal, setProposal] = useState<string | null>(null) // diff preview target
  const [explanation, setExplanation] = useState<string | null>(null)
  const promptRef = useRef<HTMLInputElement>(null)
  const editorRef = useRef<any>(null)
  const monacoRef = useRef<any>(null)

  const sourceText = Array.isArray(cell.source) ? cell.source.join('') : cell.source
  const cellError = cell.cell_type === 'code' ? errorFromOutputs(cell.outputs) : null

  // Live code-font updates broadcast from the notebook header.
  useEffect(() => {
    const onFont = (e: Event) => {
      const size = (e as CustomEvent).detail as number
      editorRef.current?.updateOptions?.({ fontSize: size })
    }
    window.addEventListener('pn-code-font', onFont)
    return () => window.removeEventListener('pn-code-font', onFont)
  }, [])

  // Mark the offending line/column in the editor (red squiggle + gutter) when a
  // cell errors, and clear it once the error is gone.
  useEffect(() => {
    const editor = editorRef.current
    const monaco = monacoRef.current
    if (!editor || !monaco) return
    const model = editor.getModel()
    if (!model) return
    if (cellError) {
      const { ename, line, col, friendly } = parseTraceback(cellError)
      if (line) {
        const startCol = col ?? 1
        monaco.editor.setModelMarkers(model, 'prismnote', [
          {
            startLineNumber: line,
            startColumn: startCol,
            endLineNumber: line,
            endColumn: col ? startCol + 1 : model.getLineMaxColumn(Math.min(line, model.getLineCount())),
            message: `${ename}: ${friendly}`,
            severity: monaco.MarkerSeverity.Error,
          },
        ])
        return
      }
    }
    monaco.editor.setModelMarkers(model, 'prismnote', [])
  }, [cellError])

  // Live stdout streaming for this cell (cleared when the final output lands).
  useEffect(() => {
    if (cell.cell_type !== 'code') return
    return subscribeCellStream(cell.id, (text) => setLiveOut((prev) => prev + text))
  }, [cell.id, cell.cell_type])

  const handleRun = async () => {
    setIsExecuting(true)
    setLiveOut('')
    try {
      await executeCell(cellIndex)
    } finally {
      setIsExecuting(false)
      setLiveOut('')
    }
  }

  // Context = the other code cells, so the model knows the surrounding notebook.
  const notebookContext = () =>
    (currentNotebook?.cells ?? [])
      .filter((c: any, i: number) => i !== cellIndex && c.cell_type === 'code')
      .map((c: any) => (Array.isArray(c.source) ? c.source.join('') : c.source))
      .join('\n\n')

  const openAi = () => {
    setAiOpen(true)
    setAiError(null)
    setTimeout(() => promptRef.current?.focus(), 0)
  }

  const runAiEdit = async () => {
    const instruction = aiPrompt.trim()
    if (!instruction || aiBusy) return
    setAiBusy(true)
    setAiError(null)
    try {
      const next = await aiEdit(sourceText, instruction, notebookContext())
      setProposal(next)
    } catch (e: any) {
      setAiError(e?.response?.data?.suggestion || e?.message || 'AI request failed')
    } finally {
      setAiBusy(false)
    }
  }

  const runAiFix = async () => {
    if (!cellError || aiBusy) return
    setAiOpen(true)
    setAiBusy(true)
    setAiError(null)
    try {
      const next = await aiFix(sourceText, cellError)
      setProposal(next)
    } catch (e: any) {
      setAiError(e?.response?.data?.suggestion || e?.message || 'AI request failed')
    } finally {
      setAiBusy(false)
    }
  }

  const runAiExplain = async () => {
    if (aiBusy) return
    setAiBusy(true)
    setAiError(null)
    setExplanation(null)
    try {
      setExplanation(await aiExplain(sourceText))
    } catch (e: any) {
      setAiError(e?.response?.data?.suggestion || e?.message || 'AI request failed')
    } finally {
      setAiBusy(false)
    }
  }

  // A dynamic-form widget changed: set its value in the kernel, then re-run this
  // cell so downstream logic recomputes with the new input.
  const onWidget = async (name: string, value: any) => {
    if (!currentNotebook) return
    try {
      await fetch(`/api/notebooks/${currentNotebook.id}/execute`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          cell_id: '__widget__',
          code: `prism._set(${JSON.stringify(name)}, ${JSON.stringify(value)})`,
        }),
      })
      await executeCell(cellIndex)
    } catch {
      /* ignore */
    }
  }

  const acceptProposal = () => {
    if (proposal == null) return
    updateCell(cellIndex, { source: proposal.split(/(?<=\n)/) })
    setProposal(null)
    setAiOpen(false)
    setAiPrompt('')
  }

  const rejectProposal = () => setProposal(null)

  return (
    <div className="pn-solid-bg rounded-lg border pn-bd overflow-hidden">
      <div className="flex items-center justify-between px-4 py-2 bg-[var(--pn-hover)]">
        <div className="flex items-center gap-2">
          <div className="w-5 text-center">
            {isExecuting ? (
              <span className="text-xs text-blue-400">*</span>
            ) : (
              <span className="text-xs pn-muted">[{cell.execution_count || '-'}]</span>
            )}
          </div>
          <span className="text-xs pn-faint">{cell.cell_type}</span>
        </div>
        <div className="flex gap-1">
          {cell.cell_type === 'code' && (
            <>
              {cellError && (
                <button
                  onClick={runAiFix}
                  disabled={aiBusy}
                  className="flex items-center gap-1 px-2 py-1 rounded text-xs bg-rose-500/15 text-rose-400 hover:bg-rose-500/25 disabled:opacity-50"
                  title="Fix this error with AI"
                >
                  <Wand2 size={13} /> Fix
                </button>
              )}
              <button
                onClick={openAi}
                className="p-1 rounded pn-hover text-violet-400"
                title="Edit with AI (⌘K)"
              >
                <Sparkles size={16} />
              </button>
              {isExecuting ? (
                <button
                  onClick={() => interruptKernel()}
                  className="p-1 rounded pn-hover text-rose-400"
                  title="Stop (interrupt kernel)"
                >
                  <Square size={15} fill="currentColor" />
                </button>
              ) : (
                <button
                  onClick={handleRun}
                  className="p-1 rounded pn-hover"
                  title="Shift+Enter"
                >
                  <Play size={16} />
                </button>
              )}
            </>
          )}
          <button
            onClick={() => deleteCell(cellIndex)}
            className="p-1 rounded pn-hover text-red-400"
          >
            <Trash2 size={16} />
          </button>
        </div>
      </div>

      {/* AI command bar (Cmd+K) */}
      {cell.cell_type === 'code' && aiOpen && (
        <div className="border-t pn-bd bg-violet-500/5 px-3 py-2">
          <div className="flex items-center gap-2">
            <Sparkles size={15} className="text-violet-400 shrink-0" />
            <input
              ref={promptRef}
              value={aiPrompt}
              onChange={(e) => setAiPrompt(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') { e.preventDefault(); runAiEdit() }
                if (e.key === 'Escape') { e.preventDefault(); setAiOpen(false); setProposal(null) }
              }}
              placeholder="Tell the AI how to change this cell… (e.g. add type hints, vectorize with numpy)"
              className="flex-1 bg-transparent outline-none text-sm pn-text placeholder:pn-faint"
            />
            {aiBusy ? (
              <Loader2 size={15} className="animate-spin pn-faint" />
            ) : (
              <button
                onClick={runAiEdit}
                disabled={!aiPrompt.trim()}
                className="px-2 py-0.5 rounded text-xs prism-bg text-white disabled:opacity-40"
              >
                Generate
              </button>
            )}
            <button onClick={() => { setAiOpen(false); setProposal(null) }} className="p-0.5 rounded pn-hover pn-faint" title="Close (Esc)">
              <X size={14} />
            </button>
          </div>
          {aiError && <div className="mt-1.5 text-xs text-rose-400">{aiError}</div>}
        </div>
      )}

      {/* Diff preview of an AI proposal, with accept / reject */}
      {proposal != null && (
        <div className="border-t pn-bd">
          <div className="flex items-center justify-between px-3 py-1.5 bg-violet-500/10">
            <span className="text-xs text-violet-300">AI suggestion — review the diff</span>
            <div className="flex gap-2">
              <button onClick={acceptProposal} className="flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30">
                <Check size={13} /> Accept
              </button>
              <button onClick={rejectProposal} className="flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-rose-500/15 text-rose-400 hover:bg-rose-500/25">
                <X size={13} /> Discard
              </button>
            </div>
          </div>
          <DiffEditor
            height="220px"
            language="python"
            original={sourceText}
            modified={proposal}
            theme={isDark() ? 'vs-dark' : 'light'}
            options={{
              renderSideBySide: false,
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              readOnly: true,
              fontSize: parseInt(localStorage.getItem('pn-code-size') || '16', 10),
            }}
          />
        </div>
      )}

      {cell.cell_type === 'code' && proposal == null && (
        <div className="border-t pn-bd">
          <Editor
            height="200px"
            language="python"
            value={sourceText}
            onMount={(editor, monaco) => {
              editorRef.current = editor
              monacoRef.current = monaco
              editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyK, openAi)
              registerOllamaCompletions(monaco) // ghost-text suggestions when Ollama is up
            }}
            // split *keeping* the trailing \n on each line so join('') round-trips
            // (otherwise newlines are lost and the cursor can't move to a new line)
            onChange={(val) => updateCell(cellIndex, { source: (val ?? '').split(/(?<=\n)/) })}
            theme={isDark() ? 'vs-dark' : 'light'}
            options={{
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              lineNumbers: 'on',
              fontSize: parseInt(localStorage.getItem('pn-code-size') || '16', 10),
              inlineSuggest: { enabled: true },
            }}
          />
        </div>
      )}

      {cell.cell_type === 'markdown' && (
        <div className="p-4 border-t pn-bd">
          {isEditing ? (
            <textarea
              value={sourceText}
              onChange={(e) => updateCell(cellIndex, { source: e.target.value.split(/(?<=\n)/) })}
              onBlur={() => setIsEditing(false)}
              className="w-full p-2 bg-[var(--pn-hover)] pn-text rounded font-mono text-sm"
              rows={4}
            />
          ) : (
            <div onClick={() => setIsEditing(true)} className="cursor-pointer hover:opacity-80">
              <MDPreview
                source={sourceText}
                style={{ backgroundColor: 'transparent', color: '#e5e7eb' }}
              />
            </div>
          )}
        </div>
      )}

      {/* AI explanation panel */}
      {explanation && (
        <div className="border-t pn-bd bg-violet-500/5 p-3 text-sm pn-text">
          <div className="flex items-center justify-between mb-1">
            <span className="text-xs text-violet-300 flex items-center gap-1"><Sparkles size={12} /> Explanation</span>
            <button onClick={() => setExplanation(null)} className="p-0.5 rounded pn-hover pn-faint"><X size={13} /></button>
          </div>
          <MDPreview source={explanation} style={{ backgroundColor: 'transparent', color: 'inherit', fontSize: 13 }} />
        </div>
      )}

      {isExecuting && liveOut && (
        <div className="border-t pn-bd bg-[var(--pn-hover)] p-4">
          <div className="text-[10px] uppercase tracking-wide pn-faint mb-1 flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" /> live
          </div>
          <pre className="whitespace-pre-wrap font-mono text-sm pn-text">{liveOut}</pre>
        </div>
      )}

      {cell.outputs.length > 0 && (
        <div className="border-t pn-bd bg-[var(--pn-hover)] p-4">
          {cell.outputs.map((output: any, idx: number) => (
            <Output key={idx} output={output} onWidget={onWidget} />
          ))}
          {cell.cell_type === 'code' && (
            <div className="mt-2 flex justify-end">
              <button
                onClick={runAiExplain}
                disabled={aiBusy}
                className="flex items-center gap-1 text-xs pn-faint hover:text-violet-400 disabled:opacity-50"
                title="Explain this cell with AI"
              >
                <Sparkles size={12} /> Explain
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
