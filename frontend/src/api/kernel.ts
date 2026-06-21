import axios from 'axios'

/** Interrupt the currently running cell (SIGINT -> KeyboardInterrupt). */
export async function interruptKernel(): Promise<void> {
  await axios.post('/api/kernel/interrupt')
}

/** Restart the kernel, clearing all variables/imports. */
export async function restartKernel(): Promise<void> {
  await axios.post('/api/kernel/restart')
}
