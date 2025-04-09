import { getApiBaseUrl } from '@/utils/api';

export const documentService = {
  async deleteDocument(collectionName: string, documentId: string) {
    const response = await fetch(
      `${getApiBaseUrl()}/collections/${collectionName}/documents/${documentId}`,
      { method: 'DELETE' }
    );
    return response.json();
  }
};