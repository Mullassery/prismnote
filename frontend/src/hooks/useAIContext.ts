import { create } from 'zustand'

// Shared "what the user is looking at" context, so the AI panel understands more
// than just notebook cells: the open dataset (Data Explorer) and the workspace
// files (File/Server Explorer). Panels publish here; the agent reads it.
interface DatasetCtx {
  title: string
  columns: string[]
  shape?: [number, number]
}

interface AIContextState {
  dataset: DatasetCtx | null
  workspace: string | null
  files: string[]
  setDataset: (d: DatasetCtx | null) => void
  setWorkspace: (name: string | null, files: string[]) => void
}

export const useAIContext = create<AIContextState>((set) => ({
  dataset: null,
  workspace: null,
  files: [],
  setDataset: (dataset) => set({ dataset }),
  setWorkspace: (workspace, files) => set({ workspace, files }),
}))

/** Render the environment block injected into the AI agent's system prompt. */
export function buildEnvironmentContext(): string {
  const { dataset, workspace, files } = useAIContext.getState()
  const parts: string[] = []
  // PrismNote is a local, single-user OSS build — there is no login/account.
  parts.push('Session: local workspace (open-source build, no account / login).')
  if (workspace) {
    const list = files.slice(0, 40).join(', ')
    parts.push(`Workspace folder: ${workspace}${list ? `\nFiles: ${list}${files.length > 40 ? ', …' : ''}` : ''}`)
  }
  if (dataset) {
    const shape = dataset.shape ? ` (${dataset.shape[0]}×${dataset.shape[1]})` : ''
    parts.push(`Open dataset in Data Explorer: ${dataset.title}${shape}\nColumns: ${dataset.columns.slice(0, 60).join(', ')}`)
  }
  return parts.join('\n\n')
}
