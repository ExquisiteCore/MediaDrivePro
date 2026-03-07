import { request } from './client'

export interface TranscodeTask {
  id: string
  file_id: string
  status: string
  profile: string
  progress: number
  output_key: string | null
  error_msg: string | null
  started_at: string | null
  completed_at: string | null
  created_at: string
}

export async function createTranscode(
  file_id: string,
  profile?: string,
): Promise<TranscodeTask> {
  return request<TranscodeTask>('/transcode', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ file_id, profile }),
  })
}

export async function getTranscode(id: string): Promise<TranscodeTask> {
  return request<TranscodeTask>(`/transcode/${id}`)
}

export async function listTranscodes(
  file_id: string,
): Promise<TranscodeTask[]> {
  return request<TranscodeTask[]>(`/transcode?file_id=${file_id}`)
}
