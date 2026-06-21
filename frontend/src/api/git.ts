import axios from 'axios'

export interface GitStatus {
  ok: boolean
  is_repo: boolean
  branch: string
  status: string
}
export interface GitResult {
  ok: boolean
  output: string
}

export const gitStatus = (dir: string) =>
  axios.get<GitStatus>('/api/git/status', { params: { path: dir } }).then((r) => r.data)
export const gitInit = (dir: string) =>
  axios.post<GitResult>('/api/git/init', { dir }).then((r) => r.data)
export const gitClone = (url: string, dir: string) =>
  axios.post<GitResult>('/api/git/clone', { url, dir }).then((r) => r.data)
export const gitCommit = (dir: string, message: string) =>
  axios.post<GitResult>('/api/git/commit', { dir, message }).then((r) => r.data)
export const gitPush = (dir: string, remote?: string, branch?: string) =>
  axios.post<GitResult>('/api/git/push', { dir, remote, branch }).then((r) => r.data)
export const gitPull = (dir: string) =>
  axios.post<GitResult>('/api/git/pull', { dir }).then((r) => r.data)
