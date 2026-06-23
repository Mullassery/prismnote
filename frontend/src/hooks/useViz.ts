import { create } from 'zustand'
import type { ExplorerTarget } from '../components/DataExplorer'

interface VizState {
  target: ExplorerTarget | null
  title: string
  // bumped each time something requests the Explore view, so listeners
  // (VizPane / BottomPanel) can react even if target is unchanged.
  nonce: number
  requestExplore: (target: ExplorerTarget, title: string) => void
}

export const useViz = create<VizState>((set) => ({
  target: null,
  title: '',
  nonce: 0,
  requestExplore: (target, title) => set((s) => ({ target, title, nonce: s.nonce + 1 })),
}))
