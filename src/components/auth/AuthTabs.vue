<!-- src/components/auth/AuthTabs.vue -->
<template>
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white p-6 rounded-lg w-96 shadow-xl">
        <!-- Tab Headers -->
        <div class="flex border-b mb-4">
          <button
            @click="activeTab = 'login'"
            class="px-4 py-2 -mb-px font-medium text-sm transition-colors"
            :class="activeTab === 'login' 
              ? 'border-b-2 border-blue-500 text-blue-600' 
              : 'text-gray-600 hover:text-blue-500'"
          >
            Login
          </button>
          <button
            @click="activeTab = 'register'"
            class="px-4 py-2 -mb-px font-medium text-sm transition-colors"
            :class="activeTab === 'register' 
              ? 'border-b-2 border-blue-500 text-blue-600' 
              : 'text-gray-600 hover:text-blue-500'"
          >
            Register
          </button>
        </div>
        
        <!-- Login Form -->
        <div v-if="activeTab === 'login'">
          <form @submit.prevent="handleLoginSubmit">
            <div class="mb-4">
              <label class="block text-sm font-medium mb-1">Email or Username</label>
              <input
                v-model="loginForm.identifier"
                type="text"
                required
                class="w-full p-2 border rounded-md"
                aria-label="Email or Username"
              >
            </div>
            <div class="mb-4">
              <label class="block text-sm font-medium mb-1">Password</label>
              <input
                v-model="loginForm.password"
                type="password"
                required
                class="w-full p-2 border rounded-md"
                aria-label="Password"
              >
            </div>
            <div v-if="loginError" class="mb-4 text-red-500 text-sm" role="alert">
              {{ loginError }}
            </div>
            <button
              type="submit"
              class="w-full bg-blue-500 text-white p-2 rounded-md hover:bg-blue-600 transition disabled:opacity-50"
              :disabled="isLoginLoading"
            >
              <span v-if="isLoginLoading">
                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white inline" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Logging in...
              </span>
              <span v-else>Login</span>
            </button>
          </form>
        </div>
        
        <!-- Register Form -->
        <div v-if="activeTab === 'register'">
          <form @submit.prevent="handleRegisterSubmit">
            <div class="mb-4">
              <label class="block text-sm font-medium mb-1">Username</label>
              <input 
                v-model="registerForm.username" 
                type="text" 
                required
                class="w-full p-2 border rounded-md"
              >
            </div>
            <div class="mb-4">
              <label class="block text-sm font-medium mb-1">Email</label>
              <input 
                v-model="registerForm.email" 
                type="email" 
                required
                class="w-full p-2 border rounded-md"
              >
            </div>
            <div class="mb-4">
              <label class="block text-sm font-medium mb-1">Password</label>
              <input 
                v-model="registerForm.password" 
                type="password" 
                required
                class="w-full p-2 border rounded-md"
              >
            </div>
            <div class="mb-4">
              <label class="block text-sm font-medium mb-1">Confirm Password</label>
              <input 
                v-model="registerForm.confirmPassword" 
                type="password" 
                required
                class="w-full p-2 border rounded-md"
              >
            </div>
            <div v-if="registerError" class="mb-4 text-red-500 text-sm">
              {{ registerError }}
            </div>
            <button 
              type="submit"
              class="w-full bg-blue-500 text-white p-2 rounded-md hover:bg-blue-600 transition disabled:opacity-50"
              :disabled="isRegisterLoading"
            >
              <span v-if="isRegisterLoading">Creating Account...</span>
              <span v-else>Register</span>
            </button>
          </form>
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
  
  const emit = defineEmits(['login', 'register', 'close'])
  
  // Active tab state
  const activeTab = ref('login')
  
  // Login form state
  const loginForm = ref({
    identifier: '',
    password: ''
  })
  const loginError = ref('')
  const isLoginLoading = ref(false)
  
  // Register form state
  const registerForm = ref({
    username: '',
    email: '',
    password: '',
    confirmPassword: ''
  })
  const registerError = ref('')
  const isRegisterLoading = ref(false)
  
  // Watch for authentication state changes
  watch(() => props.isAuthenticated, (newValue) => {
    if (newValue === true) {
      emit('close')
    }
  })
  
  const handleLoginSubmit = async () => {
    loginError.value = ''
    isLoginLoading.value = true
  
    // Add a small delay to show loading state
    await new Promise(resolve => setTimeout(resolve, 500))
  
    try {
      await emit('login', loginForm.value.identifier, loginForm.value.password)
      // The watch effect will handle closing on successful login
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Login failed'
  
      if (message === 'User not found') {
        loginError.value = 'Username or email not found'
      } else if (message === 'Invalid password') {
        loginError.value = `Invalid password for '${loginForm.value.identifier}'`
      } else {
        loginError.value = 'Login failed. Please check your credentials and try again.'
        console.error("Login Error:", err)
      }
    } finally {
      isLoginLoading.value = false
    }
  }
  
  const handleRegisterSubmit = async () => {
    registerError.value = ''
    
    if (registerForm.value.password !== registerForm.value.confirmPassword) {
      registerError.value = 'Passwords do not match'
      return
    }
    
    isRegisterLoading.value = true
  
    try {
      await emit('register', 
        registerForm.value.username, 
        registerForm.value.email, 
        registerForm.value.password
      )
      
      // On successful registration, switch to login tab
      activeTab.value = 'login'
      registerForm.value = { username: '', email: '', password: '', confirmPassword: '' }
    } catch (err) {
      registerError.value = err instanceof Error ? err.message : 'Registration failed'
    } finally {
      isRegisterLoading.value = false
    }
  }
  </script>