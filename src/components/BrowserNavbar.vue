<!-- src/components/BrowserNavbar.vue -->
<script setup lang="ts">
import { inject, ref, watch } from 'vue'
import { useZoom } from '@/composables/useZoom'

// Props for the component
const props = defineProps<{
  currentUrl?: string
}>()

const emit = defineEmits(['navigate', 'reload', 'back', 'forward'])

// Track URL input value
const urlInput = ref('')

// Use the zoom composable
const { zoomLevel, zoomIn, zoomOut, resetZoom } = inject('zoom')! as ReturnType<typeof useZoom>

// Update input when currentUrl prop changes
watch(() => props.currentUrl, (newUrl) => {
  if (newUrl) {
    const cleanUrl = newUrl.replace(/^app\/?/, '')
    urlInput.value = `app/${cleanUrl}`.replace(/\/$/, '')
  }
}, { immediate: true })

// Handle URL submission
function handleSubmit() {
  const cleanUrl = urlInput.value.replace(/^(app\/|https?:\/\/)/, '');
  emit('navigate', `app/${cleanUrl}`);
}

// Handle navigation actions
function handleBack() {
  emit('back')
}

function handleForward() {
  emit('forward')
}

function handleReload() {
  emit('reload')
}

function handleBookmark() {
  // Add bookmark functionality
  console.log('Bookmark added')
}
</script>

<template>
  <div class="flex items-center h-10 px-2 bg-[#3A3A3A] border-b border-gray-800">
    <!-- Navigation Controls -->
    <div class="flex items-center space-x-1 mr-2">
      <!-- Back Button -->
      <button 
        @click="handleBack"
        class="w-8 h-8 rounded flex items-center justify-center text-gray-400 hover:bg-[#4A4A4A]">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 12H5M12 19l-7-7 7-7"></path>
        </svg>
      </button>
      
      <!-- Forward Button -->
      <button 
        @click="handleForward"
        class="w-8 h-8 rounded flex items-center justify-center text-gray-400 hover:bg-[#4A4A4A]">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M5 12h14M12 5l7 7-7 7"></path>
        </svg>
      </button>
      
      <!-- Reload Button -->
      <button 
        @click="handleReload"
        class="w-8 h-8 rounded flex items-center justify-center text-gray-400 hover:bg-[#4A4A4A]">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M1 4v6h6"></path>
          <path d="M23 20v-6h-6"></path>
          <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15"></path>
        </svg>
      </button>
    </div>
    
    <!-- URL Input Field -->
    <div class="flex-1 relative">
      <div class="relative flex items-center w-full">
        <!-- Security/Site Info Icon -->
        <div class="absolute left-2 flex items-center justify-center text-gray-400">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
          </svg>
        </div>
        
        <!-- URL Input -->
        <form @submit.prevent="handleSubmit" class="w-full">
          <input 
            v-model="urlInput"
            type="text"
            placeholder="Search or enter URL"
            class="w-full h-8 bg-[#272822] text-gray-300 px-8 rounded focus:outline-none focus:ring-1 focus:ring-gray-500 text-sm"
          />
        </form>
      </div>
    </div>
    
    <!-- Action Buttons -->
    <div class="flex items-center space-x-0 p-0 ml-2">
      <!-- Zoom Controls -->
      <div class="flex space-x-0 absolute right-20 mr-1 mt-1">
        <button 
          @click="zoomOut"
          :disabled="zoomLevel <= 50"
          class="mt-0.5 w-7 h-7 rounded-full flex items-center justify-center text-gray-400 hover:bg-[#4A4A4A]"
          title="Zoom Out (Ctrl+-)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14"/>
          </svg>
        </button>

        <button 
          @click="resetZoom"
          class="mx-1 w-8 h-8 rounded-full flex items-center justify-center text-gray-400 hover:bg-[#4A4A4A]"
          title="Reset Zoom (Ctrl+0)"
        >
          <span class="text-xs">{{ zoomLevel }}%</span>
        </button>

        <button 
          @click="zoomIn"
          :disabled="zoomLevel >= 200"
          class="mt-0.5 w-7 h-7 rounded-full flex items-center justify-center text-gray-400 hover:bg-[#4A4A4A]"
          title="Zoom In (Ctrl+=)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 5v14M5 12h14"/>
          </svg>
        </button>
      </div>

      <!-- Bookmark Button -->
      <button 
        @click="handleBookmark"
        class="w-8 h-8 rounded flex items-center justify-center text-gray-400 hover:bg-[#4A4A4A]">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
        </svg>
      </button>
      
      <!-- Extensions Button (placeholder for your browser-like UI) -->
      <button class="w-8 h-8 rounded flex items-center justify-center text-gray-400 hover:bg-[#4A4A4A]">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="1"></circle>
          <circle cx="19" cy="12" r="1"></circle>
          <circle cx="5" cy="12" r="1"></circle>
        </svg>
      </button>
    </div>
  </div>
</template>