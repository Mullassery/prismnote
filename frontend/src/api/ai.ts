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

export interface AIConfigDetail {
  configured: boolean
  provider: 'ollama' | 'claude' | 'openai' | null
  ollama_url: string | null
  ollama_model: string | null
  openai_model: string | null
  claude_key_set: boolean
  openai_key_set: boolean
}

export interface AIConfigInput {
  provider: 'ollama' | 'claude' | 'openai'
  ollama_url?: string
  ollama_model?: string
  claude_api_key?: string
  openai_api_key?: string
  openai_model?: string
}

/** Read the active AI config (provider + non-secret fields; key presence only). */
export async function getAiConfig(): Promise<AIConfigDetail> {
  const res = await axios.get<AIConfigDetail>(`${API}/ai/config`)
  return res.data
}

/** Persist the AI provider config; hot-swaps the backend engine. */
export async function setAiConfig(cfg: AIConfigInput): Promise<void> {
  await axios.post(`${API}/ai/config`, cfg)
}

/** Single source of truth for the local Ollama endpoint (chat agent + inline
 *  autocomplete). Set in Settings → AI. */
export const ollamaEndpoint = (): string =>
  localStorage.getItem('pn-ollama') || 'http://localhost:11434'
