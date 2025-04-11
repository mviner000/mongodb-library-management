<!-- src/components/auth/AuthTabs.vue -->

<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white p-6 rounded-lg w-96 shadow-xl">
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
          @click="handleRegisterTabClick"
          class="px-4 py-2 -mb-px font-medium text-sm transition-colors"
          :class="activeTab === 'register'
            ? 'border-b-2 border-blue-500 text-blue-600'
            : 'text-gray-600 hover:text-blue-500'"
        >
          {{ registerUnlocked ? '🔓 Register' : '🔒 Register' }}
        </button>
      </div>

      <PinInputDialog
        v-model:visible="showPinModal"
        :correct-pin="registrationPin"
        title="Unlock Registration"
        :initially-show-pin="showPinDigits"
        @correct="handlePinSuccess"
        @cancel="handlePinCancel"
      />

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
              {{ loginLoadingText }}
            </span>
            <span v-else>Login</span>
          </button>
        </form>
      </div>

      <div v-if="activeTab === 'register' && registerUnlocked">
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
              minlength="3"
              class="w-full p-2 border rounded-md"
            >
            <p class="text-xs text-gray-500 mt-1">Minimum 3 characters with at least one number</p>
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
import { ref, watch, onMounted, nextTick } from 'vue'
// Remove unused PinInput imports from ui if they were specific to a library
// import { PinInput, PinInputInput, PinInputGroup } from '@/components/ui/pin-input'
import PinInputDialog from '@/components/PinInputDialog.vue'; // Import the new component
import { apiFetch } from '@/utils/api'
import { useToast } from '@/components/ui/toast'
const { toast } = useToast()

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
const loginLoadingText = ref('Logging in...')

// Register form state
const registerForm = ref({
  username: '',
  email: '',
  password: '',
  confirmPassword: ''
})
const registerError = ref('')
const isRegisterLoading = ref(false)

// PIN Input state
const showPinModal = ref(true)
const registrationPin = ref('000000');
const showPinDigits = ref(false);

// Registration lock state
const registerUnlocked = ref(false)

// Check registration lock status
const checkRegisterStatus = () => {
  const storedTime = sessionStorage.getItem('registerUnlocked')
  if (!storedTime) {
    registerUnlocked.value = false
    return
  }

  const unlockTime = parseInt(storedTime, 10)
  const currentTime = Date.now()
  registerUnlocked.value = (currentTime - unlockTime) <= 3600000 // 1 hour
}

// Handle register tab click
const handleRegisterTabClick = () => {
  checkRegisterStatus(); // Re-check status just in case
  if (!registerUnlocked.value) {
    // Only logic needed here is to show the modal
    showPinModal.value = true;
    return
  }
  activeTab.value = 'register'
}

// Handle successful PIN entry (emitted from PinInputDialog)
const handlePinSuccess = (enteredPin: string) => {
  console.log('Correct PIN entered:', enteredPin); // Optional logging
  const currentTime = Date.now()
  sessionStorage.setItem('registerUnlocked', currentTime.toString())
  registerUnlocked.value = true; // Update status immediately
  showPinModal.value = false // Close the modal
  activeTab.value = 'register' // Switch to register tab
}

// Handle PIN cancellation (emitted from PinInputDialog)
const handlePinCancel = () => {
    console.log('PIN entry cancelled');
    // The v-model already handles setting showPinModal to false
    // No extra action needed unless required by specific logic
}

// Watch for authentication state changes
watch(() => props.isAuthenticated, (newValue) => {
  if (newValue === true) {
    // Don't close immediately - it's now handled in handleLoginSubmit with a delay
  }
})

const handleLoginSubmit = async () => {
  loginError.value = ''
  isLoginLoading.value = true
  loginLoadingText.value = 'Logging in...'

  try {
    // Emit login event to handle API call in parent
    await emit('login', loginForm.value.identifier, loginForm.value.password)

    // Update loading text for the delay period
    loginLoadingText.value = 'Success! Redirecting...'

    // Add a 2-second delay before closing the modal
    setTimeout(() => {
      isLoginLoading.value = false
      emit('close')
    }, 2000)

  } catch (err) {
    const message = err instanceof Error ? err.message : 'Login failed'
    loginError.value = message
    toast({
      title: 'Login Failed',
      description: message,
      variant: 'destructive'
    })
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
    await apiFetch<void>('/api/auth/register', {
      method: 'POST',
      body: JSON.stringify({
        username: registerForm.value.username,
        email: registerForm.value.email,
        password: registerForm.value.password
      })
    })

    toast({
      title: 'Registration successful',
      description: 'You can now log in with your credentials',
    })

    activeTab.value = 'login'
    registerForm.value = { username: '', email: '', password: '', confirmPassword: '' }
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Registration failed'
    registerError.value = message
    toast({
      title: 'Registration Failed',
      description: message,
      variant: 'destructive'
    })
  } finally {
    isRegisterLoading.value = false
  }
}

// Initial check on component mount
onMounted(() => {
  checkRegisterStatus()
})
</script>