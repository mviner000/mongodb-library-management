<!-- src/components/HistoryPage.vue -->
<script setup lang="ts">
import { ref } from 'vue';

// Sample history data for the placeholder
const historyItems = ref([
  { 
    timestamp: '5:08 PM', 
    title: 'Google Sheets', 
    url: 'docs.google.com',
    icon: '📊'
  },
  { 
    timestamp: '4:46 PM', 
    title: 'Beginner\'s Guide to Triggering Collection Card Navigation - Claude', 
    url: 'claude.ai',
    icon: '🔥'
  },
  { 
    timestamp: '4:32 PM', 
    title: 'DeepSeek - Into the Unknown', 
    url: 'chat.deepseek.com',
    icon: '🔍'
  },
  { 
    timestamp: '4:19 PM', 
    title: 'Resolving Duplicate Browser Navbars in Split Tab View - Claude', 
    url: 'claude.ai',
    icon: '🔥'
  }
]);
</script>

<template>
  <div class="history-page">
    <div class="p-4">
      <div class="flex items-center mb-6">
        <svg class="w-6 h-6 mr-2" viewBox="0 0 24 24" fill="currentColor">
          <path d="M13 3c-4.97 0-9 4.03-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42C8.27 19.99 10.51 21 13 21c4.97 0 9-4.03 9-9s-4.03-9-9-9zm-1 5v5l4.25 2.52.77-1.28-3.52-2.09V8z"></path>
        </svg>
        <h1 class="text-2xl font-bold">History</h1>
      </div>

      <div class="mb-4 flex">
        <div class="flex-1">
          <input type="text" placeholder="Search history" class="w-full p-2 bg-gray-700 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
        </div>
        <div class="ml-4 flex">
          <button class="px-4 py-2 bg-gray-700 rounded-md mr-2">By date</button>
          <button class="px-4 py-2 bg-gray-700 rounded-md">By group</button>
        </div>
      </div>

      <div class="history-list">
        <div v-for="(item, index) in historyItems" :key="index" class="flex items-center py-2 border-b border-gray-700 hover:bg-gray-800">
          <input type="checkbox" class="mr-4">
          <div class="w-20 text-sm text-gray-400">{{ item.timestamp }}</div>
          <div class="mr-4 text-xl">{{ item.icon }}</div>
          <div class="flex-1">
            <div class="font-medium">{{ item.title }}</div>
            <div class="text-sm text-gray-400">{{ item.url }}</div>
          </div>
          <button class="p-2 hover:bg-gray-700 rounded-full">
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"></path>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-page {
  background-color: #202124;
  color: white;
  min-height: 100vh;
}
</style>