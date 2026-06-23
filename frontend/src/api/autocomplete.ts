// Ollama-powered inline code completion (ghost text) for Monaco.
// Registered once globally; only produces suggestions when Ollama is reachable.
// Throttled + cached so we don't hammer the local model on every keystroke.

import { ollamaEndpoint } from './ai'

const OLLAMA = () => ollamaEndpoint() // shared with the chat agent; set in Settings → AI
let registered = false
let cachedModel: { name: string | null; at: number } = { name: null, at: 0 }
let lastCall = 0

async function ollamaModel(): Promise<string | null> {
  // cache the model name for 30s to avoid a /tags round-trip per keystroke
  if (Date.now() - cachedModel.at < 30_000) return cachedModel.name
  try {
    const r = await fetch(`${OLLAMA()}/api/tags`)
    const d = r.ok ? await r.json() : null
    cachedModel = { name: d?.models?.[0]?.name ?? null, at: Date.now() }
  } catch {
    cachedModel = { name: null, at: Date.now() }
  }
  return cachedModel.name
}

export function registerOllamaCompletions(monaco: any) {
  if (registered) return
  registered = true

  monaco.languages.registerInlineCompletionsProvider(['python'], {
    async provideInlineCompletions(model: any, position: any) {
      // throttle: at most one request ~every 500ms
      const now = Date.now()
      if (now - lastCall < 500) return { items: [] }
      lastCall = now

      const prefix = model.getValueInRange({
        startLineNumber: 1,
        startColumn: 1,
        endLineNumber: position.lineNumber,
        endColumn: position.column,
      })
      if (!prefix.trim()) return { items: [] }

      const mdl = await ollamaModel()
      if (!mdl) return { items: [] } // Ollama not connected → no suggestions

      try {
        const res = await fetch(`${OLLAMA()}/api/generate`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            model: mdl,
            prompt:
              'You are a Python autocomplete engine inside a data-science notebook. ' +
              'Continue the code from the cursor. Output ONLY the raw continuation — ' +
              'no markdown fences, no commentary.\n\n' +
              prefix,
            stream: false,
            options: { temperature: 0.1, num_predict: 64, stop: ['\n\n', '```'] },
          }),
        })
        if (!res.ok) return { items: [] }
        const d = await res.json()
        const text: string = (d.response || '').replace(/```/g, '')
        if (!text.trim()) return { items: [] }
        return {
          items: [
            {
              insertText: text,
              range: new monaco.Range(
                position.lineNumber,
                position.column,
                position.lineNumber,
                position.column,
              ),
            },
          ],
        }
      } catch {
        return { items: [] }
      }
    },
    // Monaco requires both on the provider; missing disposeInlineCompletions
    // throws a TypeError when the editor disposes the provider.
    freeInlineCompletions() {},
    disposeInlineCompletions() {},
    handleItemDidShow() {},
  })
}
