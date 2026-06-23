import { create } from 'zustand'
import type { ExplorerTarget } from '../components/DataExplorer'

// App-wide bridge so any panel (file browser, data panel, …) can open the Data
// Explorer on a target without prop-drilling. App subscribes to `nonce`.
interface ExplorerRequestState {
  target: ExplorerTarget | null
  title: string
  nonce: number
  open: (target: ExplorerTarget, title: string) => void
}

export const useExplorerRequest = create<ExplorerRequestState>((set) => ({
  target: null,
  title: '',
  nonce: 0,
  open: (target, title) => set((s) => ({ target, title, nonce: s.nonce + 1 })),
}))

// File extensions DuckDB can read directly — used to decide whether a clicked
// file should open in the Data Explorer.
const DATA_EXTS = ['.parquet', '.pq', '.csv', '.tsv', '.json', '.ndjson', '.jsonl', '.arrow', '.feather']
export const isDataFile = (name: string) => DATA_EXTS.some((e) => name.toLowerCase().endsWith(e))
