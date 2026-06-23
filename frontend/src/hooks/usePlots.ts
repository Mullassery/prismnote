import { create } from 'zustand'

export interface Plot {
  id: string
  png?: string // base64 (no data: prefix)
  svg?: string // raw <svg> markup
  html?: string // plotly/altair/other interactive HTML
  createdAt: number
}

interface PlotsState {
  plots: Plot[]
  currentIndex: number
  /** Register a plot from a notebook output; deduped by content so re-renders
   *  of the same figure don't pile up. Returns true if it was newly added. */
  addFromOutput: (output: any) => boolean
  select: (index: number) => void
  next: () => void
  prev: () => void
  clear: () => void
}

// Cheap content hash so identical figures dedupe across re-runs.
function hash(s: string): string {
  let h = 0
  for (let i = 0; i < s.length; i++) h = (Math.imul(31, h) + s.charCodeAt(i)) | 0
  return String(h)
}

export const usePlots = create<PlotsState>((set, get) => ({
  plots: [],
  currentIndex: 0,
  addFromOutput: (output: any) => {
    const data = output?.data
    if (!data) return false
    const png: string | undefined = data['image/png']
    const svgRaw = data['image/svg+xml']
    const svg: string | undefined = Array.isArray(svgRaw) ? svgRaw.join('') : svgRaw
    const htmlRaw = data['text/html']
    // Only treat HTML as a plot if it looks like an interactive viz (plotly/vega/bokeh).
    const htmlStr: string | undefined = Array.isArray(htmlRaw) ? htmlRaw.join('') : htmlRaw
    const isVizHtml =
      htmlStr && /plotly|vega|bokeh|require\.config|data-plotly/i.test(htmlStr)
    const html = isVizHtml ? htmlStr : undefined
    if (!png && !svg && !html) return false

    const id = hash((png || '') + (svg || '') + (html || ''))
    const { plots } = get()
    if (plots.some((p) => p.id === id)) {
      // already captured — just focus it
      set({ currentIndex: plots.findIndex((p) => p.id === id) })
      return false
    }
    const plot: Plot = { id, png, svg, html, createdAt: Date.now() }
    const nextPlots = [...plots, plot]
    set({ plots: nextPlots, currentIndex: nextPlots.length - 1 })
    return true
  },
  select: (index) =>
    set((s) => ({ currentIndex: Math.max(0, Math.min(index, s.plots.length - 1)) })),
  next: () =>
    set((s) => ({ currentIndex: Math.min(s.currentIndex + 1, s.plots.length - 1) })),
  prev: () => set((s) => ({ currentIndex: Math.max(s.currentIndex - 1, 0) })),
  clear: () => set({ plots: [], currentIndex: 0 }),
}))
