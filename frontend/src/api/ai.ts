import axios from 'axios'

const API = '/api'

export interface AIResult {
  suggestion: string
}

/** Rewrite a cell's code from a natural-language instruction (Cmd+K). */
export async function aiEdit(
  code: string,
  instruction: string,
  context?: string,
): Promise<string> {
  const res = await axios.post<AIResult>(`${API}/ai/edit`, { code, instruction, context })
  return res.data.suggestion
}

/** Ask the model to fix code given its error text. Returns corrected code. */
export async function aiFix(code: string, error: string): Promise<string> {
  const res = await axios.post<AIResult>(`${API}/ai/fix`, { action: 'fix', code, error })
  return res.data.suggestion
}

/** Get a short natural-language explanation of a cell. */
export async function aiExplain(code: string): Promise<string> {
  const res = await axios.post<AIResult>(`${API}/ai/explain`, { action: 'explain', code })
  return res.data.suggestion
}

/** Whether an AI provider has been configured on the backend. */
export async function aiConfigured(): Promise<boolean> {
  try {
    const res = await axios.get<{ configured: boolean }>(`${API}/ai/config`)
    return !!res.data.configured
  } catch {
    return false
  }
}
