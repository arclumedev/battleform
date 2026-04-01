import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '../lib/api'

export interface User {
  id: string
  email: string
  display_name: string
}

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchProfile() {
    loading.value = true
    error.value = null
    try {
      const res = await api.get('/auth/profile')
      user.value = res.data as User
    } catch {
      user.value = null
    } finally {
      loading.value = false
    }
  }

  async function login(email: string, password: string) {
    loading.value = true
    error.value = null
    try {
      const res = await api.post('/auth/login', { email, password })
      user.value = res.data as User
    } catch {
      error.value = 'Invalid email or password'
      throw new Error("Request failed")
    } finally {
      loading.value = false
    }
  }

  async function register(email: string, password: string, displayName: string) {
    loading.value = true
    error.value = null
    try {
      const res = await api.post('/auth/register', { email, password, display_name: displayName })
      user.value = res.data as User
    } catch {
      error.value = 'Registration failed'
      throw new Error("Request failed")
    } finally {
      loading.value = false
    }
  }

  async function logout() {
    try {
      await api.post('/auth/logout')
    } finally {
      user.value = null
    }
  }

  return { user, loading, error, fetchProfile, login, register, logout }
})
