import { request } from './client'
import type { User } from './auth'

export async function listUsers(): Promise<User[]> {
  return request<User[]>('/admin/users')
}
