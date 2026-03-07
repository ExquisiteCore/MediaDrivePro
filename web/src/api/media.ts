import { request } from './client'

export interface MediaInfo {
  id: string
  file_id: string
  media_type: string
  title: string | null
  season: number | null
  episode: number | null
  tmdb_id: number | null
  poster_url: string | null
  overview: string | null
  year: number | null
  duration: number | null
  resolution: string | null
  created_at: string
}

export async function getMediaInfo(
  file_id: string,
): Promise<MediaInfo | null> {
  return request<MediaInfo | null>(`/media/${file_id}`)
}

export async function scanMedia(file_id: string): Promise<MediaInfo> {
  return request<MediaInfo>(`/media/${file_id}/scan`, { method: 'POST' })
}
