import { request } from './client'

export interface ApiTokenInfo {
  id: string
  name: string
  permissions: string
  expires_at: string | null
  last_used_at: string | null
  created_at: string
}

export interface ApiTokenCreated extends ApiTokenInfo {
  token: string
}

export interface CreateTokenRequest {
  name: string
  permissions?: string
  expires_at?: string
}

export async function createToken(data: CreateTokenRequest): Promise<ApiTokenCreated> {
  return request<ApiTokenCreated>('/tokens', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
}

export async function listTokens(): Promise<ApiTokenInfo[]> {
  return request<ApiTokenInfo[]>('/tokens')
}

export async function deleteToken(id: string): Promise<void> {
  return request<void>(`/tokens/${id}`, { method: 'DELETE' })
}
