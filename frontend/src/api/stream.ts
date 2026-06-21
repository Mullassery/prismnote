// Single shared WebSocket that receives live cell-output chunks and dispatches
// them to per-cell subscribers. Additive to the HTTP execute response.

type Handler = (text: string) => void
const subs = new Map<string, Set<Handler>>()
let ws: WebSocket | null = null
let reconnectTimer: ReturnType<typeof setTimeout> | null = null

function ensure() {
  if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return
  const proto = location.protocol === 'https:' ? 'wss' : 'ws'
  try {
    ws = new WebSocket(`${proto}://${location.host}/ws/notebook/live`)
  } catch {
    return
  }
  ws.onmessage = (e) => {
    try {
      const { cell_id, text } = JSON.parse(e.data)
      subs.get(cell_id)?.forEach((h) => h(text))
    } catch {
      /* ignore non-JSON frames */
    }
  }
  ws.onclose = () => {
    ws = null
    if (subs.size && !reconnectTimer) {
      reconnectTimer = setTimeout(() => {
        reconnectTimer = null
        ensure()
      }, 1500)
    }
  }
  ws.onerror = () => {
    try {
      ws?.close()
    } catch {
      /* ignore */
    }
  }
}

export function subscribeCellStream(cellId: string, handler: Handler): () => void {
  ensure()
  if (!subs.has(cellId)) subs.set(cellId, new Set())
  subs.get(cellId)!.add(handler)
  return () => {
    subs.get(cellId)?.delete(handler)
    if (subs.get(cellId)?.size === 0) subs.delete(cellId)
  }
}
