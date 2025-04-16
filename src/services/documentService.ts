// src/services/documentService.ts

import { AUTH_CONSTANTS } from '@/constants/auth'
import { getApiBaseUrl } from '@/utils/api'

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

export const documentService = {
  // Get the authentication token from localStorage
  getAuthToken(): string | null {
    return localStorage.getItem(AUTH_CONSTANTS.TOKEN_KEY)
  },
  // Create authentication headers for API requests
  getAuthHeaders(): HeadersInit {
    const token = this.getAuthToken()
    if (!token) {
      console.warn('No authentication token available')
      return {
        'Content-Type': 'application/json',
      }
    }

    return {
      'Content-Type': 'application/json',
      Authorization: `${AUTH_CONSTANTS.TOKEN_PREFIX} ${token}`,
    }
  },

  async fetchCollections(): Promise<ApiResponse<string[]>> {
    try {
      const response = await fetch(`${getApiBaseUrl()}/collections`)

      if (!response.ok) {
        throw new Error(`API returned ${response.status}: ${response.statusText}`)
      }

      return (await response.json()) as ApiResponse<string[]>
    } catch (error) {
      console.error('Error fetching collections:', error)
      return {
        success: false,
        error: `Failed to load collections: ${error}`,
      }
    }
  },

  async deleteDocument(collectionName: string, documentId: string) {
    const response = await fetch(
      `${getApiBaseUrl()}/collections/${collectionName}/documents/${documentId}`,
      { method: 'DELETE' }
    )
    return response.json()
  },

  async pinDocument(collectionName: string, documentId: string) {
    console.log(
      `documentService.pinDocument: Starting API call for ${collectionName}/${documentId}`
    )
    try {
      const url = `${getApiBaseUrl()}/collections/${collectionName}/documents/${documentId}/pin`
      console.log(`documentService.pinDocument: Sending PUT request to ${url}`)

      const response = await fetch(url, {
        method: 'PUT',
        headers: this.getAuthHeaders(),
      })

      console.log(`documentService.pinDocument: Received response with status ${response.status}`)

      if (!response.ok) {
        console.error(
          `documentService.pinDocument: HTTP error ${response.status}: ${response.statusText}`
        )
      }

      const data = await response.json()
      console.log(`documentService.pinDocument: Parsed response data`, data)
      return data
    } catch (error) {
      console.error('documentService.pinDocument: Error occurred:', error)
      throw error
    }
  },

  async unpinDocument(collectionName: string, documentId: string) {
    console.log(
      `documentService.unpinDocument: Starting API call for ${collectionName}/${documentId}`
    )
    try {
      const url = `${getApiBaseUrl()}/collections/${collectionName}/documents/${documentId}/unpin`
      console.log(`documentService.unpinDocument: Sending PUT request to ${url}`)

      const response = await fetch(url, {
        method: 'PUT',
        headers: this.getAuthHeaders(),
      })

      console.log(`documentService.unpinDocument: Received response with status ${response.status}`)

      if (!response.ok) {
        console.error(
          `documentService.unpinDocument: HTTP error ${response.status}: ${response.statusText}`
        )
      }

      const data = await response.json()
      console.log(`documentService.unpinDocument: Parsed response data`, data)
      return data
    } catch (error) {
      console.error('documentService.unpinDocument: Error occurred:', error)
      throw error
    }
  },
}
