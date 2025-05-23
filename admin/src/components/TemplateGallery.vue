<!-- src/components/TemplateGallery.vue -->
<script setup lang="ts">
  import { ref, onMounted, inject, computed } from 'vue'
  import { useRouter } from 'vue-router'
  import { Button } from '@/components/ui/button'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import HomeNavbar from './HomeNavbar.vue'
  import { documentService } from '@/services/documentService'
  import { MONGODB_CONSTANTS } from '@/constants/mongodb'

  interface Template {
    id: string
    title: string
    image: string
  }

  // Define the TabManager interface to match the structure in App.vue
  interface Tab {
    id: string
    title: string
    type: 'home' | 'collection' | 'hello'
    path: string
    collectionName?: string
    reloadCount: number
  }

  interface TabManager {
    openNewTab: (tab: Tab) => void
    addNewTab: () => void
  }

  // System collections that should be filtered by default
  const filteredCollectionNames = ref(MONGODB_CONSTANTS.SYSTEM_COLLECTIONS)
  const showFilteredCollections = ref(false)

  // Will be populated with MongoDB collections
  const allCollections = ref<Template[]>([])
  const isLoading = ref(true)
  const error = ref('')
  const router = useRouter()

  // Blank template definition
  const blankTemplate = ref<Template>({
    id: 'blank',
    title: 'Blank spreadsheet',
    image: '/templates/blank.png',
  })

  // Create a computed property for displayed templates
  const displayedTemplates = computed(() => {
    const nonFiltered = allCollections.value.filter(
      (t) => !filteredCollectionNames.value.includes(t.id)
    )
    const filtered = showFilteredCollections.value
      ? allCollections.value.filter((t) => filteredCollectionNames.value.includes(t.id))
      : []
    return [blankTemplate.value, ...nonFiltered, ...filtered]
  })

  // Inject the global event bus or tab management function with proper typing
  const tabManager = inject<TabManager>('tabManager')

  // Fetch collections from the API server using the service
  async function fetchCollections() {
    isLoading.value = true
    error.value = ''

    try {
      // Use the documentService instead of direct fetch
      const responseData = await documentService.fetchCollections()

      if (!responseData.success) {
        throw new Error(responseData.error || 'Unknown error occurred')
      }

      const collections = responseData.data || []

      // Convert collections to template format
      allCollections.value = collections.map((collection) => ({
        id: collection,
        title: collection.charAt(0).toUpperCase() + collection.slice(1),
        image: `/templates/collection.png`, // Using a placeholder image
      }))
    } catch (err) {
      console.error('Error fetching collections:', err)
      error.value = `Failed to load collections: ${err}`
      allCollections.value = []
    } finally {
      isLoading.value = false
    }
  }

  // Handle template selection - navigate to collection route
  const handleTemplateSelect = (templateId: string) => {
    router.replace(`/collection/${templateId}`)
  }

  // Handle opening template in new tab
  const handleOpenInNewTab = (templateId: string) => {
    // Use the emitter or tab manager to create a new tab
    if (tabManager) {
      // Create a new tab with unique ID and set its path to the collection route
      const newTabId = `tab-${Date.now()}`
      const path = `/collection/${templateId}`
      const title = templateId.charAt(0).toUpperCase() + templateId.slice(1)

      // Tell the parent to create a new tab
      tabManager.openNewTab({
        id: newTabId,
        title: title,
        type: 'collection',
        path: path,
        collectionName: templateId,
        reloadCount: 0,
      })
    } else {
      // Fallback: just navigate in current tab if tab manager isn't available
      router.push(`/collection/${templateId}`)
    }
  }

  onMounted(() => {
    fetchCollections()
  })
</script>

<template>
  <HomeNavbar
    :title="'Home'"
    class="sticky top-0 z-50"
  />
  <div class="bg-gray-50 py-6 px-4">
    <div class="max-w-7xl mx-auto">
      <div class="flex justify-between items-center mb-4">
        <h2 class="text-base font-medium text-gray-700">Start a new spreadsheet</h2>
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-600">MongoDB Collections</span>
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8 text-gray-500"
            @click="fetchCollections"
          >
            <!-- Refresh icon -->
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M21 2v6h-6"></path>
              <path d="M3 12a9 9 0 0 1 15-6.7L21 8"></path>
              <path d="M3 22v-6h6"></path>
              <path d="M21 12a9 9 0 0 1-15 6.7L3 16"></path>
            </svg>
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8 text-gray-500"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle
                cx="12"
                cy="12"
                r="1"
              ></circle>
              <circle
                cx="12"
                cy="5"
                r="1"
              ></circle>
              <circle
                cx="12"
                cy="19"
                r="1"
              ></circle>
            </svg>
          </Button>
        </div>
      </div>

      <!-- System collections toggle -->
      <div class="mb-4 flex items-center space-x-2">
        <input
          type="checkbox"
          id="showFiltered"
          v-model="showFilteredCollections"
          class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
        />
        <label
          for="showFiltered"
          class="text-sm text-gray-600"
        >
          Show system collections
        </label>
      </div>

      <!-- Loading state -->
      <div
        v-if="isLoading"
        class="flex justify-center items-center py-12"
      >
        <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500"></div>
      </div>

      <!-- Error state -->
      <div
        v-else-if="error"
        class="p-4 bg-red-50 text-red-700 rounded-md mb-4"
      >
        {{ error }}
        <Button
          variant="outline"
          size="sm"
          class="ml-2"
          @click="fetchCollections"
        >
          Retry
        </Button>
      </div>

      <!-- Template grid -->
      <div
        v-else
        class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4"
      >
        <ContextMenu
          v-for="template in displayedTemplates"
          :key="template.id"
        >
          <ContextMenuTrigger>
            <div
              class="flex flex-col items-center cursor-pointer"
              @click="handleTemplateSelect(template.id)"
            >
              <div
                class="border border-gray-200 rounded-md overflow-hidden hover:border-blue-500 transition-colors w-full"
              >
                <!-- Using placeholder images until you have actual template thumbnails -->
                <div class="w-full h-32 bg-white flex items-center justify-center">
                  <!-- For blank template, show plus icon -->
                  <div
                    v-if="template.id === 'blank'"
                    class="flex flex-col items-center justify-center"
                  >
                    <svg
                      viewBox="0 0 48 48"
                      width="48"
                      height="48"
                    >
                      <path
                        fill="#F44336"
                        d="M23,11h2v26h-2z"
                      />
                      <path
                        fill="#4CAF50"
                        d="M35,23v2H13v-2z"
                      />
                      <path
                        fill="#2196F3"
                        d="M11,23v2H7.9A1.9,1.9,0,0,1,6,23.1V23z"
                      />
                      <path
                        fill="#FFC107"
                        d="M37,23v2h3.1A1.9,1.9,0,0,0,42,23.9V23z"
                      />
                    </svg>
                  </div>
                  <!-- For MongoDB collections show a placeholder -->
                  <div
                    v-else
                    class="w-full h-full flex items-center justify-center bg-gray-50"
                  >
                    <svg
                      viewBox="0 0 48 48"
                      width="48"
                      height="48"
                    >
                      <!-- MongoDB leaf logo -->
                      <path
                        fill="#69B240"
                        d="M24,4c-4.42,0-8,3.58-8,8c0,4.92,3.3,11.08,7.33,17.15C23.99,30.83,24,35,24,35v9h0.02
                        c0,0,9.83-4.33,9.98-13.42c0-0.25,0-0.51-0.02-0.78L34,30c0,0,3.58-2.17,3.58-8.17c0-2-0.67-3.67-1.67-4.92
                        c0-2.5-3.5-7.5-3.5-7.5S34,5,34,3c-1.5,0-2.5,0.75-2.5,0.75S28.5,4,24,4z"
                      />
                    </svg>
                  </div>
                </div>
              </div>
              <span class="text-xs text-gray-700 mt-1 text-center">{{ template.title }}</span>
            </div>
          </ContextMenuTrigger>
          <ContextMenuContent>
            <ContextMenuItem @click.stop="handleOpenInNewTab(template.id)">
              Open in new tab
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
      </div>
    </div>
  </div>
</template>
