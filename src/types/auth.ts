// src/types/auth.ts
export interface LoginResponse {
    token: string
  }
  
  export interface SessionCheckResponse {
    valid: boolean
  }
  
  export interface RegisterPayload {
    username: string
    email: string
    password: string
  }
  
  export interface LoginPayload {
    identifier: string
    password: string
  }
  
  export interface ApiResponse<T> {
    success: boolean
    data?: T
    error?: string
  }