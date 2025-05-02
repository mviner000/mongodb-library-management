<!-- src/components/BrowserHistoryModal.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, watch, onUnmounted } from 'vue'
import { CloseIcon, TabIcon } from '@/components/Icons'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits(['close', 'navigate'])

// History data structure
interface HistoryItem {
  tabId: string
  nameOfTheOpenedLink: string
  created_at: string
  urlLink: string
}

// Store the history data
const historyData = ref<HistoryItem[]>([])

// Group history by tabId
const groupedHistory = computed(() => {
  const grouped: Record<string, HistoryItem[]> = {}
  
  historyData.value.forEach(item => {
    if (!grouped[item.tabId]) {
      grouped[item.tabId] = []
    }
    grouped[item.tabId].push(item)
  })
  
  // Sort each group by created_at in descending order (newest first)
  Object.keys(grouped).forEach(tabId => {
    grouped[tabId].sort((a, b) => 
      new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    )
  })
  
  return grouped
})

// Load history data from sessionStorage
function loadHistoryData() {
  try {
    const historyStr = sessionStorage.getItem('browserHistory')
    if (historyStr) {
      historyData.value = JSON.parse(historyStr)
    }
  } catch (error) {
    console.error('Failed to load browser history:', error)
  }
}

// Format date to be more readable
function formatDate(dateString: string): string {
  const date = new Date(dateString)
  return date.toLocaleString()
}

// Navigate to a history item
function navigateToHistoryItem(urlLink: string) {
  emit('navigate', urlLink)
  emit('close')
}

// Clear history for a specific tab
function clearTabHistory(tabId: string) {
  historyData.value = historyData.value.filter(item => item.tabId !== tabId)
  updateSessionStorage()
}

// Clear all history
function clearAllHistory() {
  historyData.value = []
  updateSessionStorage()
}

// Update sessionStorage with current history data
function updateSessionStorage() {
  sessionStorage.setItem('browserHistory', JSON.stringify(historyData.value))
}

// Watch for changes to isOpen prop
watch(() => props.isOpen, (newVal) => {
  if (newVal) {
    // Reload history data whenever the modal is opened
    loadHistoryData()
  }
})

// Set up a storage event listener to detect changes to sessionStorage
function handleStorageChange(event: StorageEvent) {
  if (event.key === 'browserHistory' && props.isOpen) {
    loadHistoryData()
  }
}

// Load history data when component is mounted
onMounted(() => {
  loadHistoryData()
  
  // Add event listener for storage changes
  window.addEventListener('storage', handleStorageChange)
})

// Set up an interval to periodically check for updates while modal is open
let updateInterval: number | null = null

watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    // Start polling for updates when modal is open
    updateInterval = window.setInterval(() => {
      loadHistoryData()
    }, 1000) // Check every second
  } else if (updateInterval !== null) {
    // Clear interval when modal is closed
    clearInterval(updateInterval)
    updateInterval = null
  }
})

// Clean up interval on component unmount
onUnmounted(() => {
  if (updateInterval !== null) {
    clearInterval(updateInterval)
  }
  window.removeEventListener('storage', handleStorageChange)
})
</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
    <div class="bg-[#2A2A2A] rounded-lg shadow-lg w-3/4 max-w-2xl max-h-[80vh] flex flex-col">
      <!-- Modal Header -->
      <div class="flex items-center justify-between p-4 border-b border-gray-700">
        <h2 class="text-lg font-medium text-gray-200">Browser History</h2>
        <div class="flex space-x-2">
          <button 
            @click="clearAllHistory" 
            class="px-3 py-1 text-sm text-gray-300 hover:bg-[#3A3A3A] rounded"
          >
            Clear All
          </button>
          <button 
            @click="$emit('close')" 
            class="w-8 h-8 rounded flex items-center justify-center text-gray-400 hover:bg-[#4A4A4A]"
          >
            <CloseIcon />
          </button>
        </div>
      </div>
      
      <!-- Modal Content -->
      <div class="flex-1 overflow-y-auto p-4">
        <div v-if="Object.keys(groupedHistory).length === 0" class="text-center text-gray-400 py-8">
          No browsing history found
        </div>
        
        <div v-else>
          <!-- For each tab group -->
          <div 
            v-for="(items, tabId) in groupedHistory" 
            :key="tabId" 
            class="mb-6 border border-gray-700 rounded-lg overflow-hidden"
          >
            <!-- Tab Header -->
            <div class="flex items-center justify-between bg-[#3A3A3A] px-4 py-2">
              <div class="flex items-center text-gray-200">
                <TabIcon class="w-4 h-4 mr-2" />
                <span class="font-medium">{{ tabId === 'home' ? 'Home Tab' : tabId.replace('tab-', 'Tab ') }}</span>
                <span class="ml-2 text-sm text-gray-400">({{ items.length }} items)</span>
              </div>
              <button 
                @click="clearTabHistory(tabId)" 
                class="text-xs text-gray-400 hover:text-gray-200"
              >
                Clear
              </button>
            </div>
            
            <!-- Tab History Items -->
            <div class="divide-y divide-gray-700">
              <div 
                v-for="item in items" 
                :key="item.created_at" 
                class="px-4 py-2 hover:bg-[#3A3A3A] cursor-pointer" 
                @click="navigateToHistoryItem(item.urlLink)"
              >
                <div class="text-gray-300">{{ item.nameOfTheOpenedLink }}</div>
                <div class="flex items-center justify-between">
                  <div class="text-sm text-gray-500">{{ item.urlLink }}</div>
                  <div class="text-xs text-gray-500">{{ formatDate(item.created_at) }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>