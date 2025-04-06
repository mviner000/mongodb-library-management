<!-- src/components/auth/LoginModal.vue -->
<template>
    <div
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="cancelLogin"
    >
      <div class="bg-white p-6 rounded-lg w-96 shadow-xl">
        <h2 class="text-xl font-bold mb-4">Login</h2>
        <form @submit.prevent="handleSubmit">
          <div class="mb-4">
            <label class="block text-sm font-medium mb-1">Email or Username</label>
            <input
              v-model="identifier"
              type="text"
              required
              class="w-full p-2 border rounded-md"
              aria-label="Email or Username"
            >
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium mb-1">Password</label>
            <input
              v-model="password"
              type="password"
              required
              class="w-full p-2 border rounded-md"
              aria-label="Password"
            >
          </div>
          <div v-if="error" class="mb-4 text-red-500 text-sm" role="alert">
            {{ error }}
          </div>
          <button
            type="submit"
            class="w-full bg-blue-500 text-white p-2 rounded-md hover:bg-blue-600 transition disabled:opacity-50"
            :disabled="isLoading"
          >
            <span v-if="isLoading">
              <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white inline" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Logging in...
            </span>
            <span v-else>Login</span>
          </button>
        </form>
        <div class="mt-4 text-center">
          <button
            @click.prevent="$emit('show-register')"
            class="text-blue-500 hover:text-blue-700 text-sm"
          >
            Don't have an account? Create one
          </button>
        </div>
      </div>
    </div>
  </template>

  <script setup lang="ts">
  import { ref, watch } from 'vue'

  const props = defineProps({
    isAuthenticated: {
      type: Boolean,
      default: false
    }
  })

  const emit = defineEmits(['login', 'close', 'show-register'])

  const identifier = ref('')
  const password = ref('')
  const error = ref('') // Stores the error message to display
  const isLoading = ref(false) // Controls the loading state

  // Watch for authentication state changes (e.g., close modal after successful login)
  watch(() => props.isAuthenticated, (newValue) => {
    if (newValue === true) {
        // Optionally add a success message before closing
        console.log('Login successful, closing modal.');
        emit('close')
    }
  })

  const handleSubmit = async () => {
    error.value = '' // Clear previous errors
    isLoading.value = true // Set loading state to true

    // --- ADDED 2-SECOND DELAY ---
    await new Promise(resolve => setTimeout(resolve, 500)); // Artificial 2-second delay

    try {
        // Emit the login event, expecting the parent (App.vue) to handle the actual API call
        // and potentially throw an error if login fails.
        await emit('login', identifier.value, password.value)
        // If the emit('login') resolves successfully, the watch effect will handle closing the modal
    } catch (err) {
        // Handle errors thrown from the login process
        const message = err instanceof Error ? err.message : 'Login failed'

        // Set specific error messages based on the error message from the backend
        if (message === 'User not found') {
            error.value = 'Username or email not found'
        } else if (message === 'Invalid password') {
            // Include the identifier (username/email) in the password error message
            error.value = `Invalid password for '${identifier.value}'`
        } else {
            // Generic fallback error message
            error.value = 'Login failed. Please check your credentials and try again.'
            console.error("Login Error:", err); // Log the actual error for debugging
        }
    } finally {
        // Ensure loading state is turned off regardless of success or failure
        isLoading.value = false
    }
  }

  const cancelLogin = () => {
    // If the user clicks the backdrop, emit the close event
    // Only allow closing via backdrop if not authenticated maybe? Or always allow?
    // Let's allow always for now. Add !props.isAuthenticated if needed.
    emit('close')
  }
  </script>