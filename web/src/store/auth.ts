import { create } from 'zustand'
import * as authApi from '../api/auth'
import type { User } from '../api/auth'

interface AuthState {
  user: User | null
  token: string | null
  loading: boolean
  login: (username: string, password: string) => Promise<void>
  register: (username: string, email: string, password: string) => Promise<void>
  logout: () => void
  loadUser: () => Promise<void>
  updateUser: (user: User) => void
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  token: localStorage.getItem('token'),
  loading: false,

  login: async (username, password) => {
    const res = await authApi.login({ username, password })
    localStorage.setItem('token', res.access_token)
    set({ token: res.access_token, user: res.user })
  },

  register: async (username, email, password) => {
    const res = await authApi.register({ username, email, password })
    localStorage.setItem('token', res.access_token)
    set({ token: res.access_token, user: res.user })
  },

  logout: () => {
    localStorage.removeItem('token')
    set({ token: null, user: null })
  },

  loadUser: async () => {
    try {
      set({ loading: true })
      const user = await authApi.getMe()
      set({ user, loading: false })
    } catch {
      localStorage.removeItem('token')
      set({ token: null, user: null, loading: false })
    }
  },

  updateUser: (user) => set({ user }),
}))
