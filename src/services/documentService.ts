// src/services/documentService.ts

import { getApiBaseUrl } from '@/utils/api';

interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export const documentService = {
  async fetchCollections(): Promise<ApiResponse<string[]>> {
    try {
      const response = await fetch(`${getApiBaseUrl()}/collections`);
      
      if (!response.ok) {
        throw new Error(`API returned ${response.status}: ${response.statusText}`);
      }
      
      return await response.json() as ApiResponse<string[]>;
    } catch (error) {
      console.error('Error fetching collections:', error);
      return {
        success: false,
        error: `Failed to load collections: ${error}`
      };
    }
  },
  
  async deleteDocument(collectionName: string, documentId: string) {
    const response = await fetch(
      `${getApiBaseUrl()}/collections/${collectionName}/documents/${documentId}`,
      { method: 'DELETE' }
    );
    return response.json();
  }
};