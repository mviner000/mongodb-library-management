<!-- src/components/ApiServerStatus.vue -->
<template>
  <div class="px-2 bg-[#3A3A3A]">
    <Accordion type="single" collapsible>
      <AccordionItem value="api-server-controls">
        <AccordionTrigger class="">
          <div class="flex  items-center gap-2 text-[#3A3A3A]">
            <span>API Server Controls</span>
            <div v-if="isRunning" class="flex items-center gap-2 ml-2">
              <div class="w-3 h-3 rounded-full"></div>
              <span class="text-sm">Running on port {{ serverPort }}</span>
            </div>
            <div v-else class="flex items-center gap-2 ml-2">
              <div class="w-3 h-3 rounded-full"></div>
              <span class="text-sm">Not running</span>
            </div>
          </div>
        </AccordionTrigger>
        <AccordionContent>
          <div class="flex gap-4 items-center flex-wrap py-2">
            <div class="flex gap-2 items-center">
            
            <div v-if="isRunning" class="flex items-center gap-2 ml-2">
              <div class="w-3 h-3 bg-green-500 rounded-full"></div>
              <span class="text-sm text-white ">Running on port {{ serverPort }}</span>
            </div>
            <div v-else class="flex items-center gap-2 ml-2">
              <div class="w-3 h-3 bg-red-500 rounded-full"></div>
              <span class="text-sm">Not running</span>
            </div>


              <button 
                @click="startApiServer" 
                class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-blue-300"
                :disabled="isStarting || isRunning"
              >
                {{ buttonText }}
              </button>
              <button 
                @click="stopApiServer" 
                class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 disabled:bg-red-300"
                :disabled="!isRunning"
              >
                Stop Server
              </button>
            </div>
            
            <button 
              @click="showRoutesModal = true"
              class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
              :disabled="!isRunning"
            >
              List API Routes
            </button>
          </div>
          
          <div v-if="error" class="mt-2 text-red-500">{{ error }}</div>
        </AccordionContent>
      </AccordionItem>
    </Accordion>

    <Dialog v-model:open="showRoutesModal">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>API Routes</DialogTitle>
          <DialogDescription>
            Available endpoints on http://localhost:{{ serverPort }}
          </DialogDescription>
        </DialogHeader>
        <div class="max-h-[60vh] overflow-y-auto">
          <div v-for="(route, index) in apiRoutes" :key="index" class="p-2 bg-gray-100 rounded mb-2 font-mono">
            {{ route }}
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" @click="showRoutesModal = false">Close</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
  
<script setup lang="ts">
  import { ref, onMounted, onBeforeUnmount, computed, watch } from 'vue';
  import { invoke } from '@tauri-apps/api/core';

  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '@/components/ui/dialog';

  import { Button } from '@/components/ui/button';
  import {
    Accordion,
    AccordionContent,
    AccordionItem,
    AccordionTrigger,
  } from '@/components/ui/accordion';

  const serverPort = ref(3000);
  const isStarting = ref(false);
  const isRunning = ref(false);
  const error = ref('');
  const showRoutesModal = ref(false);
  const apiRoutes = ref<string[]>([]);
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

  async function stopApiServer() {
    if (!isRunning.value) return;
    
    try {
      await invoke('stop_api_server');
      isRunning.value = false;
      error.value = '';
    } catch (err) {
      error.value = `Error stopping server: ${err}`;
      console.error(err);
    }
  }

  async function loadApiRoutes() {
    try {
      apiRoutes.value = await invoke<string[]>('list_api_routes');
    } catch (err) {
      console.error('Failed to load API routes:', err);
      apiRoutes.value = [];
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

  watch(showRoutesModal, async (newVal) => {
    if (newVal) {
      await loadApiRoutes();
    }
  });
</script>