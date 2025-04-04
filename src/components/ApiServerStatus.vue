<!-- src/components/ApiServerStatus.vue -->
<template>
    <div class="p-4">
      <div class="flex gap-4 items-center">
        <button 
          @click="startApiServer" 
          class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-blue-300"
          :disabled="isStarting || isRunning"
        >
          {{ buttonText }}
        </button>
        <div v-if="isRunning" class="flex items-center gap-2">
          <div class="w-3 h-3 bg-green-500 rounded-full"></div>
          <span>Server running on port {{ serverPort }}</span>
        </div>
        <div v-else class="flex items-center gap-2">
          <div class="w-3 h-3 bg-red-500 rounded-full"></div>
          <span>Server not running</span>
        </div>
      </div>
      <div v-if="error" class="mt-2 text-red-500">{{ error }}</div>
    </div>
  </template>
  
  <script setup lang="ts">
  import { ref, onMounted, onBeforeUnmount, computed } from 'vue';
  import { invoke } from '@tauri-apps/api/core';
  
  const serverPort = ref(3000);
  const isStarting = ref(false);
  const isRunning = ref(false);
  const error = ref('');
  let checkInterval: number;
  
  const buttonText = computed(() => {
    if (isStarting.value) return 'Starting...';
    return 'Start API Server';
  });
  
  async function checkServerStatus() {
    try {
      isRunning.value = await invoke<boolean>('is_api_server_running');
    } catch (err) {
      console.error('Status check failed:', err);
      isRunning.value = false;
    }
  }
  
  onMounted(async () => {
    await checkServerStatus();
    checkInterval = window.setInterval(checkServerStatus, 1000);
  });
  
  onBeforeUnmount(() => {
    clearInterval(checkInterval);
  });
  
  async function startApiServer() {
    if (isRunning.value || isStarting.value) return;
    
    isStarting.value = true;
    error.value = '';
    
    try {
      const response = await invoke('start_api_server', { port: serverPort.value });
      console.log(response);
      await checkServerStatus();
    } catch (err) {
      error.value = `Error starting server: ${err}`;
      console.error(err);
    } finally {
      isStarting.value = false;
    }
  }
  </script>