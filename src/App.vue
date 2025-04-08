<!-- src/App.vue -->
<script setup lang="ts">
import { ref, onMounted, provide, onUnmounted, watch, markRaw, reactive, nextTick, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useZoom } from '@/composables/useZoom'
import { useRoute, useRouter } from 'vue-router'

import Toaster from '@/components/ui/toast/Toaster.vue'
import TabBar from '@/components/TabBar.vue'
import SudoPasswordModal from '@/components/SudoPasswordModal.vue'
import MongoDBStatus from '@/components/MongoDBStatus.vue'
import MongoDBOperations from '@/components/MongoDBOperations.vue'
import MongoDBDataTable from '@/components/MongoDBDataTable.vue'
import BrowserNavbar from '@/components/BrowserNavbar.vue'
import { useToast } from './components/ui/toast'
import ApiServerStatus from './components/ApiServerStatus.vue'
import TemplateGallery from './components/TemplateGallery.vue'
import HelloWorldTab from './components/HelloWorldTab.vue'
import HistoryPage from './components/HistoryPage.vue'
import AuthTabs from './components/auth/AuthTabs.vue'
import { LoginResponse, SessionCheckResponse } from './types/auth'
import { apiFetch } from './utils/api'
import ConnectionSettingsButton from './components/ConnectionSettingsButton.vue'
import ConnectionSettingsModal from './components/ConnectionSettingsModal.vue';
import ConnectionTester from './components/ConnectionTester.vue'
import ScreenWidth from './components/ScreenWidth.vue'

const route = useRoute()
const router = useRouter()

const showAuthModal = ref(true)
const authToken = ref(localStorage.getItem('token') || null)

const componentCache = ref(new Map())

function resolveComponent(tab: Tab) {
  if (componentCache.value.has(tab.id)) {
    return componentCache.value.get(tab.id)
  }
  
  let component
  
  switch (tab.type) {
    case 'home':
      if (tab.path === '/history') {
        // Special case for history
        component = markRaw(HistoryPage)
      } else {
        component = markRaw(TemplateGallery)
      }
      break
    case 'collection':
      component = markRaw(MongoDBDataTable)
      break
    case 'hello':
      component = markRaw(HelloWorldTab)
      break
    default:
      component = markRaw(TemplateGallery)
  }
  
  componentCache.value.set(tab.id, component)
  return component
}

function resolveProps(tab: Tab) {
  if (tab.type === 'collection' && tab.collectionName) {
    return { name: tab.collectionName }
  }
  return {}
}

interface Tab {
  id: string
  title: string
  type: 'home' | 'collection' | 'hello'
  content?: string
  collectionName?: string
  path: string
  reloadCount: number
}

const tabs = ref<Tab[]>([
  { id: 'home', title: 'Home', type: 'home', path: '/home', reloadCount: 0 }
])

// --- Per-Tab History State ---
const tabHistories = reactive<Record<string, string[]>>({})
const tabHistoryPositions = reactive<Record<string, number>>({})
const isNavigatingHistory = ref(false)

// Initialize history for initial tab
tabHistories['home'] = ['/home']
tabHistoryPositions['home'] = 0

// --- Navigation Controls ---
const canGoBack = computed(() => {
  if (!activeTabId.value) return false
  return (tabHistoryPositions[activeTabId.value] ?? 0) > 0
})

const canGoForward = computed(() => {
  if (!activeTabId.value) return false
  const history = tabHistories[activeTabId.value] || []
  const position = tabHistoryPositions[activeTabId.value] ?? -1
  return position < history.length - 1
})

// For split view: compute canGoBack/Forward for each tab separately
function canTabGoBack(tabId: string): boolean {
  return (tabHistoryPositions[tabId] ?? 0) > 0
}

function canTabGoForward(tabId: string): boolean {
  const history = tabHistories[tabId] || []
  const position = tabHistoryPositions[tabId] ?? -1
  return position < history.length - 1
}

function navigateHistory(direction: 'back' | 'forward', tabId: string) {
  const history = tabHistories[tabId]
  const currentPosition = tabHistoryPositions[tabId]
  if (!history || currentPosition === undefined) return

  let newPosition = currentPosition
  if (direction === 'back' && currentPosition > 0) newPosition--
  else if (direction === 'forward' && currentPosition < history.length - 1) newPosition++
  else return

  const targetPath = history[newPosition]
  tabHistoryPositions[tabId] = newPosition

  const tab = tabs.value.find(t => t.id === tabId)
  if (tab) tab.path = targetPath

  isNavigatingHistory.value = true
  
  // Only use router.push if this is the active tab (or we're not in split view)
  if (!isSplit.value || tabId === activeTabId.value) {
    router.push(targetPath).finally(() => {
      nextTick(() => { isNavigatingHistory.value = false })
    })
  } else {
    // For non-active tab in split view, just update the tab's path
    isNavigatingHistory.value = false
  }
}

function handleBack() {
  if (canGoBack.value) {
    navigateHistory('back', activeTabId.value)
  }
}

function handleForward() {
  if (canGoForward.value) {
    navigateHistory('forward', activeTabId.value)
  }
}

// Update the split-tab navigation handlers
function handleSplitTabBack(tabId: string) {
  if (canTabGoBack(tabId)) {
    navigateHistory('back', tabId)
  }
}

function handleSplitTabForward(tabId: string) {
  if (canTabGoForward(tabId)) {
    navigateHistory('forward', tabId)
  }
}

// --- Updated History Management ---
function updateHistory(tabId: string, newPath: string) {
  // Skip history updates during back/forward navigation
  if (isNavigatingHistory.value) return

  const history = tabHistories[tabId] || []
  const currentPosition = tabHistoryPositions[tabId] ?? -1

  // Don't add duplicate entries
  if (history[currentPosition] === newPath) return

  // When navigating, trim history after current position before adding new entry
  const newHistory = history.slice(0, currentPosition + 1)
  newHistory.push(newPath)

  tabHistories[tabId] = newHistory
  tabHistoryPositions[tabId] = newHistory.length - 1

  // Don't log history entry if the path is history
  if (newPath !== '/history') {
    try {
      const tab = tabs.value.find(t => t.id === tabId)
      if (tab) {
        const storedHistory = JSON.parse(sessionStorage.getItem('browserHistory') || '[]')
        storedHistory.push({
          tabId,
          nameOfTheOpenedLink: tab.title,
          created_at: new Date().toISOString(),
          urlLink: `app${newPath}`
        })
        sessionStorage.setItem('browserHistory', JSON.stringify(storedHistory))
      }
    } catch (e) {
      console.error("History update failed:", e)
    }
  }
}

// Initialize history when creating a new tab
function initTabHistory(tabId: string, initialPath: string) {
  tabHistories[tabId] = [initialPath]
  tabHistoryPositions[tabId] = 0
}


const activeTabId = ref<string>('home')
const activeTab = ref<'home' | 'settings'>('home')
const dataTableRef = ref<InstanceType<typeof MongoDBDataTable>[]>([]);
const isConnecting = ref(false)
const connectionError = ref('')
const isSplit = ref(false)
const { toast } = useToast()
const currentUrl = ref('app/home')

// Path validation helper with guard to prevent "uninitialized variable" error
function isValidPath(path: string): boolean {
  // Guard to ensure router and its routes are ready before checking
  if (!router?.getRoutes || router.getRoutes().length === 0) {
    console.warn(`isValidPath('${path}') called before router routes are fully available.`);
    // Returning false as the safest default behavior
    return false;
  }
  
  // Access routes from the router instance
  return router.getRoutes().some(route => {
    // Exact match or wildcard match
    if (route.path === path) return true;
    // Handle dynamic segments (e.g., /collection/:name)
    if (route.path.includes(':')) {
      const routeSegments = route.path.split('/');
      const pathSegments = path.split('/');
      if (routeSegments.length !== pathSegments.length) return false;

      return routeSegments.every((segment, i) => {
        return segment.startsWith(':') || segment === pathSegments[i];
      });
    }
    return false;
  });
}

// Tab management functionality
const tabManager = reactive({
  openNewTab: (tab: Tab) => {
    tabs.value.push(tab);
    activeTabId.value = tab.id;
    
    // Initialize history for the new tab
    initTabHistory(tab.id, tab.path);
    
    router.push(tab.path);
  },

  addNewTab: () => {
    const newTabId = `tab-${Date.now()}`;
    const newTab: Tab = { 
      id: newTabId, 
      title: 'New Tab', 
      type: 'home', 
      path: '/home', 
      reloadCount: 0 
    };
    tabs.value.push(newTab);
    activeTabId.value = newTabId;
    
    // Initialize history for the new tab
    initTabHistory(newTabId, '/home');
    
    if (!isSplit.value) {
      router.push('/home');
    }
  },

  closeTab: (tabId: string) => {
    handleCloseTab(tabId);
  },

  getActiveTabId: () => activeTabId.value,

  getTab: (tabId: string) => tabs.value.find(t => t.id === tabId)
});

// Provide the tab manager to child components
provide('tabManager', tabManager);

// Use the zoom composable
const { zoomLevel, zoomIn, zoomOut, resetZoom, zoomStyle } = useZoom()

provide('zoom', { zoomLevel, zoomIn, zoomOut, resetZoom, zoomStyle })

provide('isSplit', isSplit);

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

watch(() => route.path, (newPath, oldPath) => {
  // Skip history updates if in split view, or if the paths are identical
  if (isSplit.value || newPath === oldPath) return;

  // Skip if we're currently navigating through history
  if (isNavigatingHistory.value) return;

  // Update the URL display first
  currentUrl.value = `app${newPath}`;

  // Update history for the active tab
  updateHistory(activeTabId.value, newPath);

  // Find the currently active tab's data
  const currentActiveTabData = tabs.value.find(t => t.id === activeTabId.value);
  if (currentActiveTabData) {
    // Update the active tab's properties to reflect the route change
    if (currentActiveTabData.path !== newPath) {
      currentActiveTabData.path = newPath;

      // Update title and type based on the new path
      if (newPath.startsWith('/collection/')) {
        const collectionName = newPath.split('/')[2];
        currentActiveTabData.title = collectionName;
        currentActiveTabData.type = 'collection';
        currentActiveTabData.collectionName = collectionName;
      } else if (newPath === '/home') {
        currentActiveTabData.title = 'Home';
        currentActiveTabData.type = 'home';
      } else if (newPath === '/hello') {
        currentActiveTabData.title = 'Hello World';
        currentActiveTabData.type = 'hello';
      } else {
        const pathParts = newPath.split('/').filter(Boolean);
        currentActiveTabData.title = pathParts.length > 0 ? pathParts[pathParts.length - 1] : 'Page';
        const matchedRoute = router.getRoutes().find(r => r.path === newPath || (r.path.includes(':') && isValidPath(newPath)));
        if (matchedRoute?.name === 'Collection') currentActiveTabData.type = 'collection';
        else if (matchedRoute?.name === 'Home') currentActiveTabData.type = 'home';
        else if (matchedRoute?.name === 'Hello') currentActiveTabData.type = 'hello';
        else currentActiveTabData.type = 'home'; // Fallback
      }
    }
  }
}, { immediate: false });


// Keyboard shortcut handler
function handleKeyPress(e: KeyboardEvent) {
  if (e.ctrlKey && e.key.toLowerCase() === 't') {
    e.preventDefault()
    tabManager.addNewTab();
  }
  
  if (e.ctrlKey && e.key === '\\') {
    e.preventDefault()
    
    if (tabs.value.length === 2) {
      // Check if any tab is History or has app/history in path
      const hasHistoryTab = tabs.value.some(tab => {
        return tab.path === '/history' || 
               tab.path === '/app/history' || 
               tab.path?.includes('app/history') ||
               tab.title === 'History';
      });
      
      if (hasHistoryTab) {
        toast({
          title: 'Split Tab Error',
          description: 'Cannot split when History tab is open',
        });
        return;
      }
      
      // Check if any tab is Home
      const hasHomeTab = tabs.value.some(tab => {
        // Check for multiple possible ways the home tab might be identified
        return tab.path === '/home' || 
               tab.path === '/app/home' || 
               tab.path === '/' || 
               tab.type === 'home';
      });
      
      if (!hasHomeTab) {
        isSplit.value = !isSplit.value
        // Preserve both tab paths when splitting
        if (isSplit.value) {
          tabs.value.forEach(tab => {
            if (!tab.path) tab.path = '/home';
          });
        }
      } else {
        toast({
          title: 'Split Tab Error',
          description: 'Cannot split when Home tab is open',
        })
      }
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
  
  // Add new shortcut for history
  if (e.ctrlKey && e.key.toLowerCase() === 'h') {
    e.preventDefault()
    
    // Check if split view is active
    if (isSplit.value) {
      toast({
        title: 'History Navigation Error',
        description: 'History page cannot be opened in split view mode',
      });
      return;
    }
    
    // Get the current active tab
    const currentActiveTab = tabs.value.find(t => t.id === activeTabId.value);
    
    if (currentActiveTab) {
      // Update the current tab to show history
      currentActiveTab.path = '/history';
      currentActiveTab.title = 'History';
      currentActiveTab.type = 'home'; // Reuse home type since it's a placeholder
      router.push('/history');
    }
  }
}

// Tab closing logic
function handleCloseTab(tabId: string) {
  const index = tabs.value.findIndex(t => t.id === tabId)
  if (index !== -1) {
    tabs.value.splice(index, 1)
    
    // Exit split view if less than 2 tabs remain
    if (tabs.value.length < 2) {
      isSplit.value = false
    }
    
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
    // Only navigate if not in split view
    if (!isSplit.value) {
      router.push(tab.path)
    }
  }
}

// Handle adding a new tab from TabBar
function handleAddTab() {
  tabManager.addNewTab();
}

// MODIFIED: Enhanced navigation handler with renamed variable
function handleNavigation(inputUrl: string) {
  try {
    // Parse the input URL to get the path
    const url = new URL(inputUrl.startsWith('http') ? inputUrl : `http://${inputUrl}`);
    let path = url.pathname;

    // Handle app/ prefix
    if (path.startsWith('/app/')) {
      path = path.replace('/app', '');
    } else if (path.startsWith('app/')) {
      path = '/' + path.slice(4);
    } else if (path === '/') { // Handle root path if needed
      path = '/home'; // Or your default route
    }

    // Validate path
    if (!isValidPath(path)) {
      toast({ title: 'Invalid URL', description: 'Please check the entered path' });
      return;
    }

    // Always update the current active tab's path (RENAMED variable)
    const currentActiveTabData = tabs.value.find(t => t.id === activeTabId.value);
    if (currentActiveTabData) {
      // Update the active tab's details
      currentActiveTabData.path = path;
      if (path.startsWith('/collection/')) {
        const collectionName = path.split('/')[2];
        currentActiveTabData.title = collectionName;
        currentActiveTabData.type = 'collection';
        currentActiveTabData.collectionName = collectionName;
      } else if (path === '/home') {
        currentActiveTabData.title = 'Home';
        currentActiveTabData.type = 'home';
      } else if (path === '/hello') { 
        currentActiveTabData.title = 'Hello World';
        currentActiveTabData.type = 'hello';
      } else {
        // Attempt to derive title from path, or set a generic one
        const pathParts = path.split('/').filter(Boolean);
        currentActiveTabData.title = pathParts.length > 0 ? pathParts[pathParts.length - 1] : 'Page';
        // Determine type based on route definition if possible, otherwise default
        const matchedRoute = router.getRoutes().find(r => r.path === path || (r.path.includes(':') && isValidPath(path))); 
        if (matchedRoute?.name === 'Collection') currentActiveTabData.type = 'collection';
        else if (matchedRoute?.name === 'Home') currentActiveTabData.type = 'home';
        else if (matchedRoute?.name === 'Hello') currentActiveTabData.type = 'hello';
        else currentActiveTabData.type = 'home'; // Fallback type
      }
      // Push the new path to the router, this will update the view
      router.push(path);
    }
  } catch (error) {
    console.error("URL Parsing/Navigation Error:", error);
    // Handle invalid URL format (simplified - remove existing tab check here too)
    const cleanPath = '/' + inputUrl.replace(/^(app\/|https?:\/\/)/, '');
    if (isValidPath(cleanPath)) {
      const path = cleanPath;

      // Update current tab (RENAMED variable)
      const currentActiveTabData = tabs.value.find(t => t.id === activeTabId.value);
      if (currentActiveTabData) {
        currentActiveTabData.path = path;
        // Update title/type as done in the try block...
        if (path.startsWith('/collection/')) {
          const collectionName = path.split('/')[2];
          currentActiveTabData.title = collectionName;
          currentActiveTabData.type = 'collection';
          currentActiveTabData.collectionName = collectionName;
        } else if (path === '/home') {
          currentActiveTabData.title = 'Home';
          currentActiveTabData.type = 'home';
        } else if (path === '/hello') {
          currentActiveTabData.title = 'Hello World';
          currentActiveTabData.type = 'hello';
        } else {
          const pathParts = path.split('/').filter(Boolean);
          currentActiveTabData.title = pathParts.length > 0 ? pathParts[pathParts.length - 1] : 'Page';
          const matchedRoute = router.getRoutes().find(r => r.path === path || (r.path.includes(':') && isValidPath(path)));
          if (matchedRoute?.name === 'Collection') currentActiveTabData.type = 'collection';
          else if (matchedRoute?.name === 'Home') currentActiveTabData.type = 'home';
          else if (matchedRoute?.name === 'Hello') currentActiveTabData.type = 'hello';
          else currentActiveTabData.type = 'home'; // Fallback
        }
        router.push(path);
      }
    } else {
      toast({ title: 'Invalid URL', description: 'Please check the entered path' });
    }
  }
}

// Tab-specific navigation handler
function handleTabNavigation(inputUrl: string, tabId: string) {
  const tab = tabs.value.find(t => t.id === tabId);
  if (!tab) return;

  try {
    // Process URL to get the path
    const url = new URL(inputUrl.startsWith('http') ? inputUrl : `http://${inputUrl}`);
    let path = url.pathname;

    // Handle app/ prefix
    if (path.startsWith('/app/')) {
      path = path.replace('/app', '');
    } else if (path.startsWith('app/')) {
      path = '/' + path.slice(4);
    }

    // Validate path
    if (!isValidPath(path)) {
      toast({ title: 'Invalid URL', description: 'Please check the entered path' });
      return;
    }

    // Update tab properties
    const oldPath = tab.path;
    tab.path = path;
    tab.reloadCount++; // Force re-render

    // Update history for this specific tab
    if (path !== oldPath) {
      updateHistory(tabId, path);
    }

    // Update title and type based on path
    if (path.startsWith('/collection/')) {
      const collectionName = path.split('/')[2];
      tab.title = collectionName;
      tab.type = 'collection';
      tab.collectionName = collectionName;
    } else if (path === '/home') {
      tab.title = 'Home';
      tab.type = 'home';
    } else if (path === '/hello') {
      tab.title = 'Hello World';
      tab.type = 'hello';
    } else {
      const pathParts = path.split('/').filter(Boolean);
      tab.title = pathParts.length > 0 ? pathParts[pathParts.length - 1] : 'Page';
      const matchedRoute = router.getRoutes().find(r => r.path === path || (r.path.includes(':') && isValidPath(path)));
      if (matchedRoute?.name === 'Collection') tab.type = 'collection';
      else if (matchedRoute?.name === 'Home') tab.type = 'home';
      else if (matchedRoute?.name === 'Hello') tab.type = 'hello';
      else tab.type = 'home'; // Fallback
    }
  } catch (error) {
    // Handle invalid URL format
    const cleanPath = inputUrl.replace(/^app\/?/, '');
    if (isValidPath('/' + cleanPath)) {
      const path = '/' + cleanPath;
      const oldPath = tab.path;
      
      // Update tab properties
      tab.path = path;
      tab.reloadCount++; // Force re-render

      // Update history for this specific tab
      if (path !== oldPath) {
        updateHistory(tabId, path);
      }

      // Update title and type based on path
      if (path.startsWith('/collection/')) {
        const collectionName = path.split('/')[2];
        tab.title = collectionName;
        tab.type = 'collection';
        tab.collectionName = collectionName;
      } else if (path === '/home') {
        tab.title = 'Home';
        tab.type = 'home';
      } else if (path === '/hello') {
        tab.title = 'Hello World';
        tab.type = 'hello';
      } else {
        const pathParts = path.split('/').filter(Boolean);
        tab.title = pathParts.length > 0 ? pathParts[pathParts.length - 1] : 'Page';
        const matchedRoute = router.getRoutes().find(r => r.path === path || (r.path.includes(':') && isValidPath(path)));
        if (matchedRoute?.name === 'Collection') tab.type = 'collection';
        else if (matchedRoute?.name === 'Home') tab.type = 'home';
        else if (matchedRoute?.name === 'Hello') tab.type = 'hello';
        else tab.type = 'home'; // Fallback
      }
    } else {
      toast({ title: 'Invalid URL', description: 'Please check the entered path' });
    }
  }
}

// Implementation of reload functionality
function handleReload() {
  const currentActiveTabData = tabs.value.find(t => t.id === activeTabId.value);
  if (currentActiveTabData) {
    // Increment reload counter to force component re-render
    currentActiveTabData.reloadCount++;
  }
}

// Tab-specific reload handler
function handleTabReload(tabId: string) {
  const tab = tabs.value.find(t => t.id === tabId);
  if (tab) {
    tab.reloadCount++;
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

// Lifecycle hooks
onMounted(() => {
  window.addEventListener('keydown', handleKeyPress);
  autoConnectMongoDB();
  
  currentUrl.value = `app${route.path}`;
  
  // Make sure we're starting with just one tab
  tabs.value = [{ id: 'home', title: 'Home', type: 'home', path: '/home', reloadCount: 0 }];
  activeTabId.value = 'home';
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyPress);
});

onMounted(async () => {
  // Start with the auth modal hidden during authentication check
  showAuthModal.value = false

  if (authToken.value) {
    try {
      const response = await apiFetch<SessionCheckResponse>('/api/auth/check-session', {
        method: 'POST',
        body: JSON.stringify({ token: authToken.value })
      })
      
      if (!response.valid) {
        authToken.value = null
        localStorage.removeItem('token')
        showAuthModal.value = true
      }
    } catch (error) {
      console.error('Session check failed:', error)
      authToken.value = null
      localStorage.removeItem('token')
      showAuthModal.value = true
    }
  } else {
    // No token found, show auth modal
    showAuthModal.value = true
  }
})

// login handler
const handleLogin = async (identifier: string, password: string) => {
  try {
    const response = await apiFetch<LoginResponse>('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify({ identifier, password })
    })

    // Update parent state
    authToken.value = response.token
    localStorage.setItem('token', response.token)
    
    // No need to close the modal here - AuthTabs will do it after delay
    // showAuthModal.value = false // This is now handled by AuthTabs after delay

    toast({
      title: 'Login Successful',
      description: 'Welcome back!',
    })
  } catch (error) {
    console.error('Login failed:', error)
    toast({
      title: 'Login Failed',
      description: error instanceof Error ? error.message : 'Invalid credentials',
      variant: 'destructive'
    })
    throw error
  }
}

const handleRegister = async (username: string, email: string, password: string) => {
  try {
    await apiFetch<void>('/api/auth/register', {
      method: 'POST',
      body: JSON.stringify({ username, email, password })
    })
    
    toast({
      title: 'Registration successful',
      description: 'You can now log in with your credentials',
    })
  } catch (error) {
    console.error('Registration failed:', error)
    throw new Error(error instanceof Error ? error.message : 'Registration failed')
  }
}

const handleLogout = () => {
  // Navigate to home
  router.push('/home');
  // Clear authentication state
  authToken.value = null;
  localStorage.removeItem('token');
  // Show auth modal
  showAuthModal.value = true;
};

// Simplified close handler
const handleCloseAuthModal = () => {
  // Only allow closing if authenticated
  if (authToken.value) {
    showAuthModal.value = false
  }
}

// this to the data section
const showConnectionSettingsModal = ref(false)

// Add/modify the connection settings handler
const handleConnectionSettings = () => {
  showConnectionSettingsModal.value = true
}

// this handler for saving connection settings
const handleSaveConnectionSettings = (_ipAddress: string) => {
  // Close the modal
  showConnectionSettingsModal.value = false
  
  // Force app reload to apply the new API base URL
  // This will reload all components and re-establish connections
  window.location.reload()
}

// this handler for closing connection settings modal
const handleCloseConnectionSettings = () => {
  showConnectionSettingsModal.value = false
}

</script>

<template>
  <Toaster />
  <ConnectionTester />
  <ScreenWidth />
  <ConnectionSettingsModal
      :is-open="showConnectionSettingsModal"
      @close="handleCloseConnectionSettings"
      @save="handleSaveConnectionSettings"
    />

  <AuthTabs
    v-if="showAuthModal"
    :isAuthenticated="!!authToken"
    @login="handleLogin"
    @register="handleRegister"
    @close="handleCloseAuthModal"
  />
  <div class="flex flex-col min-h-screen">
    <ApiServerStatus />

    <!-- Top-level TabBar (non-split view) -->
    <TabBar 
      v-if="!isSplit"
      :tabs="tabs"
      :active-tab-id="activeTabId"
      @close-tab="handleCloseTab"
      @tab-click="handleTabClick"
      @add-tab="handleAddTab"
    />


    <!-- Main BrowserNavbar (only when not in split view) -->
    <BrowserNavbar 
      v-if="!isSplit"
      :current-url="currentUrl" 
      :can-go-back="canGoBack"
      :can-go-forward="canGoForward"
      @navigate="handleNavigation"
      @reload="handleReload"
      @back="handleBack"
      @forward="handleForward"
      @logout="handleLogout"
    />
    
    <div class="flex flex-1">
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
            <!-- Split View Panes -->
            <div v-if="isSplit && tabs.length === 2" 
              ref="containerRef"
              class="flex h-full relative"
              :class="{ 'select-none': isDragging }">
              
              <!-- Left Pane -->
              <div 
                class="h-full overflow-auto" 
                :style="{ width: `${leftWidth}%` }"
                @click="activeTabId = tabs[0].id"
              >
                <TabBar 
                  :tabs="[tabs[0]]"
                  :active-tab-id="activeTabId"
                  :show-add-button="false"
                  @close-tab="handleCloseTab"
                  @tab-click="handleTabClick"
                />
                <BrowserNavbar 
                  :current-url="`app${tabs[0].path}`"
                  :can-go-back="canTabGoBack(tabs[0].id)"
                  :can-go-forward="canTabGoForward(tabs[0].id)"
                  @navigate="(url) => handleTabNavigation(url, tabs[0].id)"
                  @reload="() => handleTabReload(tabs[0].id)"
                  @back="() => handleSplitTabBack(tabs[0].id)"
                  @forward="() => handleSplitTabForward(tabs[0].id)"
                  @logout="handleLogout"
                />
                <div class="h-full">
                  <component 
                    :is="resolveComponent(tabs[0])" 
                    :key="`${tabs[0].id}-${tabs[0].reloadCount}`"
                    v-bind="resolveProps(tabs[0])"
                  />
                </div>
              </div>

              <!-- Resize Handle -->
              <div class="w-1 bg-gray-200 hover:bg-blue-400 cursor-col-resize relative"
                @mousedown="startResize"
                :class="{ 'bg-blue-400': isDragging }"
              ></div>

              <!-- Right Pane -->
              <div 
                class="h-full overflow-auto" 
                :style="{ width: `${100 - leftWidth}%` }"
                @click="activeTabId = tabs[1].id"
              >
                <TabBar 
                  :tabs="[tabs[1]]"
                  :active-tab-id="activeTabId"
                  :show-add-button="false"
                  @close-tab="handleCloseTab"
                  @tab-click="handleTabClick"
                />
                <BrowserNavbar 
                  :current-url="`app${tabs[1].path}`"
                  :can-go-back="canTabGoBack(tabs[1].id)"
                  :can-go-forward="canTabGoForward(tabs[1].id)"
                  @navigate="(url) => handleTabNavigation(url, tabs[1].id)"
                  @reload="() => handleTabReload(tabs[1].id)"
                  @back="() => handleSplitTabBack(tabs[1].id)"
                  @forward="() => handleSplitTabForward(tabs[1].id)"
                  @logout="handleLogout"
                />
                <div class="h-full">
                  <component 
                    :is="resolveComponent(tabs[1])" 
                    :key="`${tabs[1].id}-${tabs[1].reloadCount}`"
                    v-bind="resolveProps(tabs[1])"
                  />
                </div>
              </div>
            </div>

            <div v-else>
              <!-- Add reloadCount to the key to force re-rendering when reload happens -->
              <router-view 
                :key="activeTabId + '-' + (tabs.find(t => t.id === activeTabId)?.reloadCount || 0)"
              />
            </div>
          </div>
        </div>
      </main>
    </div>
    
    <ConnectionSettingsButton @click="handleConnectionSettings" />
  </div>
</template>