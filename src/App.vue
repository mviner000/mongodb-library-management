<!-- src/App.vue -->
<script setup lang="ts">
import { ref, onMounted, provide, computed, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useZoom } from '@/composables/useZoom'
import { useRoute, useRouter } from 'vue-router'

import Toaster from '@/components/ui/toast/Toaster.vue'
import TemplateGalleryNavbar from '@/components/TemplateGalleryNavbar.vue'
import Sidebar from '@/components/Sidebar.vue'
import TabBar from '@/components/TabBar.vue'
import SudoPasswordModal from '@/components/SudoPasswordModal.vue'
import MongoDBStatus from '@/components/MongoDBStatus.vue'
import MongoDBOperations from '@/components/MongoDBOperations.vue'
import MongoDBDataTable from '@/components/MongoDBDataTable.vue'
import BrowserNavbar from '@/components/BrowserNavbar.vue'
import MongoDBTableNavbar from '@/components/MongoDBTableNavbar.vue'
import { useToast } from './components/ui/toast'
import ApiServerStatus from './components/ApiServerStatus.vue'

const route = useRoute()
const router = useRouter()

interface Tab {
  id: string
  title: string
  type: 'home' | 'collection' | 'hello'
  content?: string
  collectionName?: string
  path: string
}

const tabs = ref<Tab[]>([
  { id: 'home', title: 'Home', type: 'home', path: '/home' }
])

const activeTabType = computed(() => {
  const activeTab = tabs.value.find(t => t.id === activeTabId.value)
  return activeTab?.type || 'home'
})

const activeTabId = ref<string>('home')
const activeTab = ref<'home' | 'settings'>('home')
const dataTableRef = ref<InstanceType<typeof MongoDBDataTable>[]>([]);
const isConnecting = ref(false)
const connectionError = ref('')
const isSidebarOpen = ref(false)
const isSplit = ref(false)
const { toast } = useToast()

// Use the zoom composable
const { zoomLevel, zoomIn, zoomOut, resetZoom, zoomStyle } = useZoom()

provide('zoom', { zoomLevel, zoomIn, zoomOut, resetZoom, zoomStyle })

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

// Router integration
function syncTabsWithRoute() {
  const currentPath = route.path
  
  // Check if tab for current route already exists
  const existingTab = tabs.value.find(tab => tab.path === currentPath)
  
  if (existingTab) {
    // If exists, just activate it
    activeTabId.value = existingTab.id
  } else {
    // Create a new tab based on the route
    let newTab: Tab
    
    if (currentPath === '/home') {
      newTab = { id: 'home', title: 'Home', type: 'home', path: '/home' }
    } else if (currentPath.startsWith('/collection/')) {
      const collectionName = route.params.name as string
      newTab = { 
        id: `col-${Date.now()}`, 
        title: collectionName, 
        type: 'collection', 
        collectionName, 
        path: currentPath 
      }
    } else if (currentPath === '/hello') {
      newTab = { 
        id: `hello-${Date.now()}`, 
        title: 'Hello World', 
        type: 'hello', 
        content: 'Hello World',
        path: '/hello' 
      }
    } else {
      // Default fallback
      newTab = { id: 'home', title: 'Home', type: 'home', path: '/home' }
      router.push('/home')
    }
    
    tabs.value.push(newTab)
    activeTabId.value = newTab.id
  }
}

// Watch for route changes
watch(() => route.path, () => {
  syncTabsWithRoute()
}, { immediate: true })

// Existing computed properties
const navbarTitle = computed(() => {
  return tabs.value.find(t => t.id === activeTabId.value)?.title || 'Library Manager'
})

const showSearch = computed(() => {
  return tabs.value.find(t => t.id === activeTabId.value)?.type === 'home'
})

// Keyboard shortcut handler
function handleKeyPress(e: KeyboardEvent) {
  if (e.ctrlKey && e.key.toLowerCase() === 't') {
    e.preventDefault()
    router.push('/hello')
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
  router.push(`/collection/${templateId}`)
}

function handleOpenInNewTab(templateId: string) {
  // Open in new tab by navigating to the path
  router.push(`/collection/${templateId}`)
}

// Tab closing logic
function handleCloseTab(tabId: string) {
  const index = tabs.value.findIndex(t => t.id === tabId)
  if (index !== -1) {
    const closingTab = tabs.value[index]
    tabs.value.splice(index, 1)
    
    // If closing active tab, go to another tab
    if (activeTabId.value === tabId) {
      const newActiveTab = tabs.value[tabs.value.length - 1]
      activeTabId.value = newActiveTab?.id || 'home'
      
      // Navigate to the new active tab's path
      if (newActiveTab) {
        router.push(newActiveTab.path)
      } else {
        router.push('/home')
      }
    }
  }
}

// Tab clicking handler
function handleTabClick(tabId: string) {
  const tab = tabs.value.find(t => t.id === tabId)
  if (tab) {
    activeTabId.value = tabId
    router.push(tab.path)
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
  router.back()
}

function handleForward() {
  router.forward()
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
  syncTabsWithRoute() // Initial sync
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyPress)
})
</script>

<template>
  <Toaster />
  <div class="flex flex-col min-h-screen">
    <ApiServerStatus />

    <TabBar 
      :tabs="tabs"
      :active-tab-id="activeTabId"
      @close-tab="handleCloseTab"
      @tab-click="handleTabClick"
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
        <!-- Apply zoom styling to this wrapper div -->
        <div :style="zoomStyle">
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

          <!-- Use router-view for tab content -->
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
                    :showSearch="tabs[0].type === 'home'"
                    class="sticky top-0 z-50"
                  />
                  <router-view v-if="tabs[0].path === route.path" />
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
                    :showSearch="tabs[1].type === 'home'"
                    class="fixed top-0 z-50"
                  />
                  <router-view v-if="tabs[1].path === route.path" />
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
                class="fixed top-0 z-50"
              />
              
              <router-view />
            </div>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>