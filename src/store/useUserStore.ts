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
    console.log('Stored token:', localStorage.getItem(AUTH_CONSTANTS.TOKEN_KEY))
    try {
      console.log('Fetching user data from /api/auth/me')
      loading.value = true
      error.value = null

      const token = localStorage.getItem(AUTH_CONSTANTS.TOKEN_KEY)
      console.log(`Auth token available: ${!!token}`, token ? `(${token.substring(0, 6)}...)` : '')

      if (!token) {
        console.error('No authentication token found')
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

      console.log('User data fetched successfully:', response)
      user.value = response
    } catch (err: any) {
      console.error('Error fetching user data:', err)
      if (err.response) {
        console.error(`Response status: ${err.response.status}`)
        try {
          const errorData = await err.response.json()
          console.error('Response data:', errorData)
          error.value = errorData.error || `Failed with status ${err.response.status}`
        } catch (parseError) {
          console.error('Could not parse error response:', parseError)
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
    console.log('Clearing user data')
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
