<!-- src/components/ConnectionTester.vue -->
<script setup lang="ts">
import { ref, onMounted, watchEffect } from 'vue'
import { getApiBaseUrl } from '../utils/api'
import { useToast } from '../components/ui/toast'

const connectionFailed = ref(false)
const isChecking = ref(false)
const apiBaseUrl = ref('')
const { toast } = useToast()

const checkConnection = async () => {
  if (isChecking.value) return
  
  isChecking.value = true
  connectionFailed.value = false
  apiBaseUrl.value = getApiBaseUrl()
  
  try {
    // Use a simple HEAD request to check if the server is reachable
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), 5000) // 5 second timeout
    
    const response = await fetch(`${apiBaseUrl.value}/api/health`, {
      method: 'HEAD',
      signal: controller.signal
    })
    
    clearTimeout(timeoutId)
    
    if (!response.ok) {
      connectionFailed.value = true
      toast({
        title: 'Connection Error',
        description: `Failed to connect to API server at ${apiBaseUrl.value}`,
        variant: 'destructive'
      })
    }
  } catch (error) {
    console.error('Connection test failed:', error)
    connectionFailed.value = true
    toast({
      title: 'Connection Error',
      description: `Failed to connect to API server at ${apiBaseUrl.value}`,
      variant: 'destructive'
    })
  } finally {
    isChecking.value = false
  }
}

// Initial check on component mount
onMounted(() => {
  checkConnection()
})

// Re-check when API base URL changes
watchEffect(() => {
  const newApiBaseUrl = getApiBaseUrl()
  if (newApiBaseUrl !== apiBaseUrl.value) {
    apiBaseUrl.value = newApiBaseUrl
    checkConnection()
  }
})

// Setup periodic checks (every 30 seconds)
onMounted(() => {
  const intervalId = setInterval(() => {
    checkConnection()
  }, 30000)
  
  // Clean up interval on component unmount
  return () => clearInterval(intervalId)
})
</script>

<template>
  <div v-if="connectionFailed" class="w-full bg-red-500 text-white p-2 text-center font-medium sticky top-0 z-50">
    Connection to API server at {{ apiBaseUrl }} failed. Please check your network or API server status.
  </div>
</template>