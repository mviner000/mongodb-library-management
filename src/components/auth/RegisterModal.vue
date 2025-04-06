<!-- src/components/auth/RegisterModal.vue -->
<template>
    <div 
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="cancelRegister"
    >
      <div class="bg-white p-6 rounded-lg w-96 shadow-xl">
        <h2 class="text-xl font-bold mb-4">Create Account</h2>
        <form @submit.prevent="handleSubmit">
          <!-- Add username field -->
          <div class="mb-4">
            <label class="block text-sm font-medium mb-1">Username</label>
            <input 
              v-model="username" 
              type="text" 
              required
              class="w-full p-2 border rounded-md"
              @input="console.log('Username input:', username)"
            >
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium mb-1">Email</label>
            <input 
              v-model="email" 
              type="email" 
              required
              class="w-full p-2 border rounded-md"
              @input="console.log('Email input:', email)"
            >
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium mb-1">Password</label>
            <input 
              v-model="password" 
              type="password" 
              required
              class="w-full p-2 border rounded-md"
              @input="console.log('Password input changed')"
            >
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium mb-1">Confirm Password</label>
            <input 
              v-model="confirmPassword" 
              type="password" 
              required
              class="w-full p-2 border rounded-md"
              @input="console.log('Confirm password input changed')"
            >
          </div>
          <div v-if="error" class="mb-4 text-red-500 text-sm">
            {{ error }}
          </div>
          <button 
            type="submit"
            class="w-full bg-blue-500 text-white p-2 rounded-md hover:bg-blue-600 transition"
            :disabled="isLoading"
            @click="console.log('Registration submission initiated')"
          >
            <span v-if="isLoading">Creating Account...</span>
            <span v-else>Register</span>
          </button>
          <div class="mt-4 text-center">
            <button 
              @click.prevent="showLogin"
              class="text-blue-500 hover:text-blue-700 text-sm"
              @click="console.log('Navigate to login requested')"
            >
              Already have an account? Login here
            </button>
          </div>
        </form>
      </div>
    </div>
  </template>
  
  <script setup lang="ts">
  import { ref } from 'vue'
  
  const emit = defineEmits(['register', 'close', 'show-login'])
  
  const username = ref('')
  const email = ref('')
  const password = ref('')
  const confirmPassword = ref('')
  const error = ref('')
  const isLoading = ref(false)
  
  console.log('RegisterModal component mounted')
  
  const handleSubmit = async () => {
    console.log('Registration submission started')
    console.log('Form data:', {
      username: username.value,
      email: email.value,
      password: password.value.length > 0 ? '***' : 'empty'
    })
  
    if (password.value !== confirmPassword.value) {
      console.warn('Password mismatch error')
      error.value = 'Passwords do not match'
      return
    }
  
    error.value = ''
    isLoading.value = true
  
    try {
      console.log('Attempting registration...')
      await emit('register', username.value, email.value, password.value)
      console.log('Registration successful')
      emit('close')
    } catch (err) {
      console.error('Registration error:', err)
      error.value = err instanceof Error ? err.message : 'Registration failed'
    } finally {
      isLoading.value = false
    }
  }
  
  const showLogin = () => {
    console.log('Switching to login view')
    emit('show-login')
  }
  
  const cancelRegister = () => {
    console.log('Registration modal closed')
    emit('close')
  }
  </script>