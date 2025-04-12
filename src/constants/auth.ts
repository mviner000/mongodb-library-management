// src/constants/auth.ts
export const AUTH_CONSTANTS = {
    TOKEN_KEY: 'auth-token', // can be changed to 'auth-token' or any other name
    TOKEN_PREFIX: 'Bearer',
    SESSION_EXPIRY: 7 * 24 * 60 * 60 * 1000, // 7 days in milliseconds (optional)
  }