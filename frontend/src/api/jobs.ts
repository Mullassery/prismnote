import axios from 'axios'

export interface Schedule {
  kind: 'manual' | 'interval' | 'daily'
  minutes?: number
  time?: string
}

export interface JobRun {
  started_at: string
  finished_at: string
  status: string
  cells_ok: number
  cells_failed: number
  log: string
}

export interface JobSummary {
  id: string
  name: string
  schedule: Schedule
  created_at: string
  last_run: string | null
  last_status: string | null
  cells: number
  runs: number
}

export async function listJobs(): Promise<JobSummary[]> {
  const res = await axios.get<{ jobs: JobSummary[] }>('/api/jobs')
  return res.data.jobs
}

export async function createJob(name: string, cells: string[], schedule: Schedule) {
  await axios.post('/api/jobs', { name, cells, schedule })
}

export async function runJob(id: string): Promise<JobRun> {
  const res = await axios.post<JobRun>(`/api/jobs/${id}/run`)
  return res.data
}

export async function deleteJob(id: string) {
  await axios.delete(`/api/jobs/${id}`)
}

export async function getJob(id: string) {
  const res = await axios.get(`/api/jobs/${id}`)
  return res.data
}

export async function airflowDag(id: string): Promise<{ dag: string; filename: string }> {
  const res = await axios.get(`/api/jobs/${id}/airflow-dag`)
  return res.data
}

