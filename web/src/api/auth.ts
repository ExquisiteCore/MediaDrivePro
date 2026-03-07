import { request } from './client'

export interface User {
  id: string
  username: string
  email: string
  role: string
  avatar: string | null
  storage_quota: number
  storage_used: number
  created_at: string
}

export interface TokenResponse {
  access_token: string
  user: User
}

export interface LoginRequest {
  username: string
  password: string
}

export interface RegisterRequest {
  username: string
  email: string
  password: string
}

export async function login(data: LoginRequest): Promise<TokenResponse> {
  return request<TokenResponse>('/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
}

export async function register(data: RegisterRequest): Promise<TokenResponse> {
  return request<TokenResponse>('/auth/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
}

export async function getMe(): Promise<User> {
  return request<User>('/auth/me')
}

export async function uploadAvatar(file: File): Promise<User> {
  const form = new FormData()
  form.append('avatar', file)
  return request<User>('/auth/avatar', {
    method: 'POST',
    body: form,
  })
}
