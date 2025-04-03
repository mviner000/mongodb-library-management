<!-- src/App.vue -->
<script setup lang="ts">
import { ref, onMounted, provide, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import SheetsNavbar from '@/components/SheetsNavbar.vue'
import Sidebar from '@/components/Sidebar.vue'
import TabBar from '@/components/TabBar.vue'
import TemplateGallery from '@/components/TemplateGallery.vue'
import SudoPasswordModal from '@/components/SudoPasswordModal.vue'
import MongoDBStatus from '@/components/MongoDBStatus.vue'
import MongoDBOperations from '@/components/MongoDBOperations.vue'
import MongoDBDataTable from '@/components/MongoDBDataTable.vue'
import HelloWorldTab from '@/components/HelloWorldTab.vue'

interface Tab {
  id: string
  title: string
  type: 'default' | 'collection' | 'hello'
  content?: string
  collectionName?: string
}

const tabs = ref<Tab[]>([
  { id: 'default', title: 'untitled', type: 'default' }
])
const activeTabId = ref<string>('default')
const activeTab = ref<'home' | 'settings'>('home')
const dataTableRef = ref<InstanceType<typeof MongoDBDataTable> | null>(null)
const isConnecting = ref(false)
const connectionError = ref('')
const isSidebarOpen = ref(false)

// Computed properties
const navbarTitle = computed(() => {
  const activeTab = tabs.value.find(t => t.id === activeTabId.value)
  return activeTab?.title || 'Library Manager'
})

const showSearch = computed(() => {
  const activeTab = tabs.value.find(t => t.id === activeTabId.value)
  return activeTab?.type === 'default'
})

// Keyboard shortcut handler
function handleKeyPress(e: KeyboardEvent) {
  if (e.ctrlKey && e.key.toLowerCase() === 't') {
    e.preventDefault()
    const newTabId = `tab-${Date.now()}`
    tabs.value.push({
      id: newTabId,
      title: 'New Tab',
      type: 'hello',
      content: 'Hello World'
    })
    activeTabId.value = newTabId
  }
}

// Template selection handler
function handleTemplateSelected(templateId: string) {
  createCollectionTab(templateId);
}

function handleOpenInNewTab(templateId: string) {
  createCollectionTab(templateId);
}

function createCollectionTab(templateId: string) {
  const newTabId = `col-${Date.now()}`;
  tabs.value.push({
    id: newTabId,
    title: templateId,
    type: 'collection',
    collectionName: templateId
  });
  activeTabId.value = newTabId;
}

// Tab closing logic
function handleCloseTab(tabId: string) {
  const index = tabs.value.findIndex(t => t.id === tabId)
  if (index !== -1) {
    tabs.value.splice(index, 1)
    if (activeTabId.value === tabId) {
      activeTabId.value = tabs.value[tabs.value.length - 1]?.id || 'default'
    }
  }
}

// MongoDB connection logic
async function autoConnectMongoDB() {
  isConnecting.value = true;
  connectionError.value = '';
  
  try {
    const isInstalled = await invoke<boolean>('is_mongodb_installed');
    if (isInstalled) {
      await invoke('connect_mongodb', {
        connectionString: 'mongodb://localhost:27017'
      });
      
      await new Promise(resolve => setTimeout(resolve, 500));
      
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

// Document action handler
function handleDocumentAction(event: { type: string, collectionName: string }) {
  if (dataTableRef.value) {
    dataTableRef.value.fetchDocuments();
    dataTableRef.value.fetchCollections();
  }
}

// Sidebar state
provide('sidebarState', {
  isOpen: isSidebarOpen,
  toggle: () => isSidebarOpen.value = !isSidebarOpen.value,
  close: () => isSidebarOpen.value = false
})

// Lifecycle hooks
onMounted(() => {
  window.addEventListener('keydown', handleKeyPress)
  autoConnectMongoDB()
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyPress)
})
</script>

<template>
  <div class="flex flex-col min-h-screen">
    <TabBar 
      :tabs="tabs"
      :active-tab-id="activeTabId"
      @close-tab="handleCloseTab"
      @tab-click="activeTabId = $event"
    />

    <SheetsNavbar 
      :title="navbarTitle" 
      :showSearch="showSearch"
    />
    
    <div class="flex flex-1">
      <Sidebar :activeTab="activeTab" @selectTab="activeTab = $event" />
      
      <main class="flex-1 overflow-auto">
        <SudoPasswordModal />
        
        <div v-if="connectionError" class="mb-4 p-3 bg-red-100 text-red-700 rounded-md">
          {{ connectionError }}
        </div>

        <template v-if="activeTab === 'home'">
          <div v-for="tab in tabs" :key="tab.id">
            <div v-if="tab.id === activeTabId" class="h-full">
              <HelloWorldTab 
                v-if="tab.type === 'hello'" 
                :content="tab.content" 
              />
              
              <template v-else-if="tab.type === 'collection'">
                <div class="p-2">
                  <Button variant="ghost" size="sm" class="mb-2" @click="handleCloseTab(tab.id)">
                    <span class="flex items-center">
                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1">
                        <path d="M19 12H5M12 19l-7-7 7-7"></path>
                      </svg>
                      Back to Collections
                    </span>
                  </Button>
                </div>
                <MongoDBDataTable 
                  ref="dataTableRef" 
                  :selected-collection="tab.collectionName"
                />
              </template>

              <TemplateGallery 
                v-else-if="tab.type === 'default'" 
                @templateSelected="handleTemplateSelected"
                @openInNewTab="handleOpenInNewTab"
              />
            </div>
          </div>
        </template>

        <template v-else>
          <div class="p-4">
            <h1 class="text-2xl font-bold mb-6 text-center">MongoDB Database Manager</h1>
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