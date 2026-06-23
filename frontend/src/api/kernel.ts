import axios from 'axios'

/** Interrupt the currently running cell (SIGINT -> KeyboardInterrupt). */
export async function interruptKernel(): Promise<void> {
  await axios.post('/api/kernel/interrupt')
}

/** Restart the kernel, clearing all variables/imports. */
export async function restartKernel(): Promise<void> {
  await axios.post('/api/kernel/restart')
}

export interface KernelVariable {
  name: string
  type: string
  preview?: string
  shape?: number[]
  len?: number
}

/** Snapshot of user-defined variables in the live kernel namespace. */
export async function listVariables(): Promise<KernelVariable[]> {
  const r = await axios.get<{ variables: KernelVariable[] }>('/api/kernel/variables')
  return r.data.variables ?? []
}
