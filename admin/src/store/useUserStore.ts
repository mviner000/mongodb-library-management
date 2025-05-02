// src/store/userStore.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { apiFetch } from '@/utils/api'
import { AUTH_CONSTANTS } from '@/constants/auth'

interface User {
  id: string
  username: string
  email: string
}

export const useUserStore = defineStore('user', () => {
  // State
  const user = ref<User | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Actions
  const fetchUser = async () => {
    try {
      loading.value = true
      error.value = null

      const token = localStorage.getItem(AUTH_CONSTANTS.TOKEN_KEY)

      if (!token) {
        error.value = 'No authentication token found'
        return
      }

      const response = await apiFetch<User>('/api/auth/me', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `${AUTH_CONSTANTS.TOKEN_PREFIX} ${token}`,
        },
      })

      user.value = response
    } catch (err: any) {
      if (err.response) {
        try {
          const errorData = await err.response.json()
          error.value = errorData.error || `Failed with status ${err.response.status}`
        } catch (parseError) {
          error.value = `Failed with status ${err.response.status}`
        }
      } else {
        error.value = err.message || 'Failed to fetch user data'
      }
      user.value = null
    } finally {
      loading.value = false
    }
  }

  const clearUser = () => {
    user.value = null
    error.value = null
  }

  return {
    user,
    loading,
    error,
    fetchUser,
    clearUser,
  }
})
