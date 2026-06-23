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
  claude_model: string | null
  openai_model: string | null
  claude_key_set: boolean
  openai_key_set: boolean
}

export interface AIConfigInput {
  provider: 'ollama' | 'claude' | 'openai'
  ollama_url?: string
  ollama_model?: string
  claude_api_key?: string
  claude_model?: string
  openai_api_key?: string
  openai_model?: string
}

/** Provider-agnostic chat for the agent panel (Claude/OpenAI via backend). */
export async function aiChat(
  messages: { role: string; content: string }[],
  system?: string,
): Promise<string> {
  const res = await axios.post<{ reply: string }>(`${API}/ai/chat`, { messages, system })
  return res.data.reply
}

// Selectable models per cloud provider (latest Claude + common OpenAI).
export const CLAUDE_MODELS = ['claude-opus-4-8', 'claude-sonnet-4-6', 'claude-haiku-4-5-20251001', 'claude-fable-5']
export const OPENAI_MODELS = ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-4']

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
