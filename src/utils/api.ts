// src/utils/api.ts
// Helper function to get the API base URL
export function getApiBaseUrl(): string {
  const savedIp = localStorage.getItem('API_BASE_IP');
  return `http://${savedIp || 'localhost'}:3000`;
}
interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

export async function apiFetch<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const API_BASE = getApiBaseUrl();
  
  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  const result: ApiResponse<T> = await response.json();
  
  if (!result.success) {
    throw new Error(result.error || 'Unknown API error');
  }

  return result.data as T;
}

// Function to test the connection to the API server
export async function testConnection(timeout = 5000): Promise<boolean> {
  const API_BASE = getApiBaseUrl();
  
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeout);
    
    const response = await fetch(`${API_BASE}/api/health`, {
      method: 'HEAD',
      signal: controller.signal
    });
    
    clearTimeout(timeoutId);
    return response.ok;
  } catch (error) {
    console.error('Connection test failed:', error);
    return false;
  }
}