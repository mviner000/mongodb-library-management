<!-- src/App.vue -->
<script setup lang="ts">
import { ref, onMounted, provide } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import SheetsNavbar from '@/components/SheetsNavbar.vue'
import Sidebar from '@/components/Sidebar.vue'
import TemplateGallery from '@/components/TemplateGallery.vue'
import SudoPasswordModal from '@/components/SudoPasswordModal.vue'
import MongoDBStatus from '@/components/MongoDBStatus.vue'
import MongoDBOperations from '@/components/MongoDBOperations.vue'
import MongoDBDataTable from '@/components/MongoDBDataTable.vue'

const activeTab = ref<'home' | 'settings'>('home')
const dataTableRef = ref<InstanceType<typeof MongoDBDataTable> | null>(null)
const isConnecting = ref(false)
const connectionError = ref('')
const isSidebarOpen = ref(false)
const showTemplateGallery = ref(true)
const selectedTemplate = ref<string | null>(null)

// Provide sidebar state to components
provide('sidebarState', {
  isOpen: isSidebarOpen,
  toggle: () => isSidebarOpen.value = !isSidebarOpen.value,
  close: () => isSidebarOpen.value = false
})

// Auto-connect to MongoDB on startup
async function autoConnectMongoDB() {
  isConnecting.value = true;
  connectionError.value = '';
  
  try {
    const isInstalled = await invoke<boolean>('is_mongodb_installed');
    if (isInstalled) {
      await invoke('connect_mongodb', {
        connectionString: 'mongodb://localhost:27017'
      });
      console.log('Successfully connected to MongoDB');
      
      // Add small delay to ensure connection is ready
      await new Promise(resolve => setTimeout(resolve, 500));
      
      // Refresh data table after connection
      if (dataTableRef.value) {
        dataTableRef.value.fetchDocuments();
      }
    }
  } catch (error) {
    connectionError.value = `Failed to connect to MongoDB: ${error}`;
  } finally {
    isConnecting.value = false;
  }
}

// Handle document operations
function handleDocumentAction(event: { type: string, collectionName: string }) {
  // Refresh the data table when document operations occur
  if (dataTableRef.value) {
    dataTableRef.value.fetchDocuments();
    dataTableRef.value.fetchCollections();
  }
}

// Handle template selection
function handleTemplateSelected(templateId: string) {
  selectedTemplate.value = templateId;
  showTemplateGallery.value = false;
  console.log(`Selected template: ${templateId}`);
  // Here you would normally load the template data or create a new spreadsheet
}

// Try to connect on component mount
onMounted(() => {
  autoConnectMongoDB();
});
</script>

<template>
  <div class="flex flex-col min-h-screen">
    <SheetsNavbar />
    <div class="flex flex-1">
      <Sidebar :activeTab="activeTab" @selectTab="activeTab = $event" />
      
      <main class="flex-1 overflow-auto">
        <SudoPasswordModal />
        
        <div v-if="connectionError" class="mb-4 p-3 bg-red-100 text-red-700 rounded-md">
          {{ connectionError }}
        </div>

        <template v-if="activeTab === 'home'">
          <!-- Show template gallery when no template is selected -->
          <TemplateGallery v-if="showTemplateGallery" @templateSelected="handleTemplateSelected" />
          
          <!-- Show data table when a template is selected -->
          <MongoDBDataTable v-else ref="dataTableRef" />
        </template>

        <template v-else>
          <div class="p-4">
            <h1 class="text-2xl font-bold mb-6 text-center">MongoDB Database Manager</h1>
            
            <!-- Center components horizontally with full width -->
            <div class="flex flex-col items-center w-full gap-6">
              <MongoDBStatus class="w-full max-w-3xl" @connection-status-changed="autoConnectMongoDB" />
              <MongoDBOperations class="w-full max-w-3xl" @document-action="handleDocumentAction" />
            </div>
          </div>
        </template>
      </main>
    </div>
  </div>
</template>