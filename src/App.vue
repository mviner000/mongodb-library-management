<!-- src/App.vue -->
<script setup lang="ts">
import { ref, onMounted, provide, computed, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

import Toaster from '@/components/ui/toast/Toaster.vue'
import TemplateGalleryNavbar from '@/components/TemplateGalleryNavbar.vue'
import Sidebar from '@/components/Sidebar.vue'
import TabBar from '@/components/TabBar.vue'
import TemplateGallery from '@/components/TemplateGallery.vue'
import SudoPasswordModal from '@/components/SudoPasswordModal.vue'
import MongoDBStatus from '@/components/MongoDBStatus.vue'
import MongoDBOperations from '@/components/MongoDBOperations.vue'
import MongoDBDataTable from '@/components/MongoDBDataTable.vue'
import BrowserNavbar from '@/components/BrowserNavbar.vue'
import MongoDBTableNavbar from '@/components/MongoDBTableNavbar.vue'
import HelloWorldTab from '@/components/HelloWorldTab.vue'
import { useToast } from './components/ui/toast'

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

const activeTabType = computed(() => {
  const activeTab = tabs.value.find(t => t.id === activeTabId.value)
  return activeTab?.type || 'default'
})

const activeTabId = ref<string>('default')
const activeTab = ref<'home' | 'settings'>('home')
const dataTableRef = ref<InstanceType<typeof MongoDBDataTable>[]>([]);
const isConnecting = ref(false)
const connectionError = ref('')
const isSidebarOpen = ref(false)
const isSplit = ref(false)
const { toast } = useToast()

// Split pane resizing
const leftWidth = ref(50)
const isDragging = ref(false)
const containerRef = ref<HTMLElement | null>(null)

function startResize(_e: MouseEvent) {
  isDragging.value = true
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', stopResize)
}

function handleMouseMove(e: MouseEvent) {
  if (!isDragging.value || !containerRef.value) return
  
  const containerRect = containerRef.value.getBoundingClientRect()
  const newWidth = ((e.clientX - containerRect.left) / containerRect.width) * 100
  leftWidth.value = Math.max(20, Math.min(80, newWidth))
}

function stopResize() {
  isDragging.value = false
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', stopResize)
}

watch(isSplit, (newVal) => {
  if (newVal) leftWidth.value = 50
})

// Existing computed properties
const navbarTitle = computed(() => {
  return tabs.value.find(t => t.id === activeTabId.value)?.title || 'Library Manager'
})

const showSearch = computed(() => {
  return tabs.value.find(t => t.id === activeTabId.value)?.type === 'default'
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
  
  if (e.ctrlKey && e.key === '\\') {
    e.preventDefault()
    if (tabs.value.length === 2) {
      isSplit.value = !isSplit.value
    } else if (tabs.value.length > 2) {
      toast({
        title: 'Split Tab Error',
        description: 'Split Tab only works for 2 tabs opened',
      })
    } else {
      toast({
        title: 'Split Tab Error',
        description: 'Not enough tabs to split',
      })
    }
  }
}

// Template selection handler
function handleTemplateSelected(templateId: string) {
  // Open in the current tab if it's the default tab
  const currentTab = tabs.value.find(t => t.id === activeTabId.value);
  
  if (currentTab?.type === 'default') {
    // Update the current tab instead of creating a new one
    const updatedTabs = tabs.value.map(tab => {
      if (tab.id === activeTabId.value) {
        return {
          ...tab,
          title: templateId,
          type: 'collection' as const, // Use 'as const' to enforce literal type
          collectionName: templateId
        };
      }
      return tab;
    });
    
    // Explicitly type the assignment to satisfy TypeScript
    tabs.value = updatedTabs as Tab[];
  } else {
    // Create a new tab if we're not on the default tab
    createCollectionTab(templateId);
  }
}

function handleOpenInNewTab(templateId: string) {
  // Always create a new tab for context menu "open in new tab"
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

// browser nav functions
const currentUrl = ref('')

// Add these empty function stubs that your colleague can implement later
function handleNavigation(_url: string) {
  // Your colleague will implement this
}

function handleReload() {
  // Your colleague will implement this
}

function handleBack() {
  // Your colleague will implement this
}

function handleForward() {
  // Your colleague will implement this
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
      // No need to manually call fetchDocuments here
    }
  } catch (error) {
    connectionError.value = `Failed to connect to MongoDB: ${error}`;
  } finally {
    isConnecting.value = false;
  }
}

// Document action handler
function handleDocumentAction(_event: { type: string, collectionName: string }) {
  dataTableRef.value.forEach(comp => {
    comp.fetchDocuments();
    comp.fetchCollections();
  });
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
  <Toaster />
  <div class="flex flex-col min-h-screen">
    <TabBar 
      :tabs="tabs"
      :active-tab-id="activeTabId"
      @close-tab="handleCloseTab"
      @tab-click="activeTabId = $event"
    />

    <BrowserNavbar 
      :current-url="currentUrl" 
      @navigate="handleNavigation"
      @reload="handleReload"
      @back="handleBack"
      @forward="handleForward"
    />
    
    <div class="flex flex-1">
      <Sidebar :activeTab="activeTab" @selectTab="activeTab = $event" />
      
      <main class="flex-1 overflow-auto">
        <SudoPasswordModal />
        
        <div v-if="connectionError" class="mb-4 p-3 bg-red-100 text-red-700 rounded-md">
          {{ connectionError }}
        </div>

        <!-- Show settings when activeTab is 'settings' -->
        <div v-if="activeTab === 'settings'">
          <div class="p-4">
            <h1 class="text-2xl font-bold mb-6 text-center">MongoDB Database Manager</h1>
            <div class="flex flex-col items-center w-full gap-6">
              <MongoDBStatus class="w-full max-w-3xl" @connection-status-changed="autoConnectMongoDB" />
              <MongoDBOperations class="w-full max-w-3xl" @document-action="handleDocumentAction" />
            </div>
          </div>
        </div>

        <!-- Only show tab content when not in settings -->
        <div v-else>
          <div v-if="isSplit && tabs.length === 2" 
               ref="containerRef"
               class="flex h-full relative"
               :class="{ 'select-none': isDragging }">
            <!-- Left Pane -->
            <div class="h-full overflow-auto" :style="{ width: `${leftWidth}%` }">
              <div class="h-full">
                <MongoDBTableNavbar 
                  v-if="tabs[0].type === 'collection'"
                  :title="tabs[0].title"
                  class="sticky top-0 z-50"
                />
                <TemplateGalleryNavbar
                  v-else
                  :title="tabs[0].title"
                  :showSearch="tabs[0].type === 'default'"
                  class="sticky top-0 z-50"
                />
                <div v-for="(tab, index) in tabs" :key="tab.id">
                  <div v-if="index === 0" class="h-full">
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
              </div>
            </div>

            <div class="w-1 bg-gray-200 hover:bg-blue-400 cursor-col-resize relative"
              @mousedown="startResize"
              :class="{ 'bg-blue-400': isDragging }"
            ></div>

            <!-- Right Pane -->
            <div class="h-full overflow-auto" :style="{ width: `${100 - leftWidth}%` }">
              <div class="h-full">
                <MongoDBTableNavbar 
                  v-if="tabs[1].type === 'collection'"
                  :title="tabs[1].title"
                  class="sticky top-0 z-50"
                />
                <TemplateGalleryNavbar
                  v-else
                  :title="tabs[1].title"
                  :showSearch="tabs[1].type === 'default'"
                  class="sticky top-0 z-50"
                />
                <div v-for="(tab, index) in tabs" :key="tab.id">
                  <div v-if="index === 1" class="h-full">
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
              </div>
            </div>
          </div>

          <div v-else>
            <MongoDBTableNavbar 
              v-if="activeTabType === 'collection'"
              :title="navbarTitle" 
              class="sticky top-0 z-50"
            />
            <TemplateGalleryNavbar
              v-else
              :title="navbarTitle" 
              :showSearch="showSearch"
              class="sticky top-0 z-50"
            />
            
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
          </div>

          <div v-if="!activeTabId">
            <div class="p-4">
              <h1 class="text-2xl font-bold mb-6 text-center">MongoDB Database Manager</h1>
              <div class="flex flex-col items-center w-full gap-6">
                <MongoDBStatus class="w-full max-w-3xl" @connection-status-changed="autoConnectMongoDB" />
                <MongoDBOperations class="w-full max-w-3xl" @document-action="handleDocumentAction" />
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>