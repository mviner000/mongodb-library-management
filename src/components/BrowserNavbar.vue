<!-- src/components/BrowserNavbar.vue -->
<script setup lang="ts">
  import { inject, ref, watch, onMounted, onBeforeUnmount } from 'vue'
  import { useZoom } from '@/composables/useZoom'
  import {
    ShieldIcon,
    PasswordIcon,
    UserIcon,
    EditIcon,
    SyncIcon,
    SettingsIcon,
    LogoutIcon,
    MinusIcon,
    PlusIcon,
    BookmarkIcon,
    DotsIcon,
    BackIcon,
    ForwardIcon,
    ReloadIcon,
    HistoryIcon,
  } from '@/components/Icons'
  import BrowserHistoryModal from '@/components/BrowserHistoryModal.vue'
  import { useUserStore } from '@/store/useUserStore'

  // Props for the component
  const props = defineProps<{
    currentUrl?: string
    canGoBack?: boolean
    canGoForward?: boolean
  }>()

  const emit = defineEmits(['navigate', 'reload', 'back', 'forward', 'logout'])

  // Track URL input value
  const urlInput = ref('')

  // Use the zoom composable
  const { zoomLevel, zoomIn, zoomOut, resetZoom } = inject('zoom')! as ReturnType<typeof useZoom>

  // Track profile dropdown state
  const showProfileDropdown = ref(false)

  // Track history modal state
  const showHistoryModal = ref(false)

  // Get the user store
  const userStore = useUserStore()

  // Fetch user when component mounts if logged in
  onMounted(async () => {
    await userStore.fetchUser()
  })

  // Update handleLogout to clear user
  function handleLogout() {
    emit('logout')
    userStore.clearUser()
    showProfileDropdown.value = false
  }

  // Update input when currentUrl prop changes
  watch(
    () => props.currentUrl,
    (newUrl) => {
      if (newUrl) {
        const cleanUrl = newUrl.replace(/^app\/?/, '')
        urlInput.value = `app/${cleanUrl}`.replace(/\/$/, '')
      }
    },
    { immediate: true }
  )

  // Handle URL submission
  function handleSubmit() {
    const cleanUrl = urlInput.value.replace(/^(app\/|https?:\/\/)/, '')
    emit('navigate', `app/${cleanUrl}`)
  }

  // Handle navigation actions with proper history management
  function handleBack() {
    emit('back')
  }

  function handleForward() {
    emit('forward')
  }

  function handleReload() {
    emit('reload')
  }

  function handleBookmark() {
    // Add bookmark functionality
    console.log('Bookmark added')
  }

  function toggleProfileDropdown() {
    showProfileDropdown.value = !showProfileDropdown.value
  }

  function toggleHistoryModal() {
    showHistoryModal.value = !showHistoryModal.value
  }

  // Close dropdown when clicking outside
  function closeDropdown(event: MouseEvent) {
    const target = event.target as HTMLElement
    if (!target.closest('.profile-dropdown') && !target.closest('.profile-button')) {
      showProfileDropdown.value = false
    }
  }

  // Add global click handler to close dropdown when clicking outside
  function handleGlobalClick(event: MouseEvent) {
    closeDropdown(event)
  }

  // Add and remove event listener for global click
  onMounted(() => {
    document.addEventListener('click', handleGlobalClick)
  })

  onBeforeUnmount(() => {
    document.removeEventListener('click', handleGlobalClick)
  })

  // Handle navigation from history modal
  function handleHistoryNavigation(url: string) {
    emit('navigate', url)
    showHistoryModal.value = false // Close modal after navigation
  }

  // Fixed getUserInitial function to handle null user safely
  function getUserInitial(): string {
    if (userStore.user && userStore.user.username && userStore.user.username.length > 0) {
      return userStore.user.username[0].toUpperCase()
    }
    return 'U'
  }
</script>

<template>
  <div class="flex items-center h-10 px-2 bg-[#3A3A3A] border-b border-gray-800">
    <!-- Navigation Controls -->
    <div class="flex items-center space-x-1 mr-2">
      <!-- Back Button -->
      <button
        @click="handleBack"
        :disabled="!canGoBack"
        class="w-8 h-8 rounded flex items-center justify-center hover:bg-[#4A4A4A]"
        :class="canGoBack ? 'text-gray-300' : 'text-gray-600 cursor-not-allowed'"
      >
        <BackIcon />
      </button>

      <!-- Forward Button -->
      <button
        @click="handleForward"
        :disabled="!canGoForward"
        class="w-8 h-8 rounded flex items-center justify-center hover:bg-[#4A4A4A]"
        :class="canGoForward ? 'text-gray-300' : 'text-gray-600 cursor-not-allowed'"
      >
        <ForwardIcon />
      </button>

      <!-- Reload Button -->
      <button
        @click="handleReload"
        class="w-8 h-8 rounded flex items-center justify-center text-gray-300 hover:bg-[#4A4A4A]"
      >
        <ReloadIcon />
      </button>
    </div>

    <!-- URL Input Field -->
    <div class="flex-1 relative">
      <div class="relative flex items-center w-full">
        <!-- Security/Site Info Icon -->
        <div class="absolute left-2 flex items-center justify-center text-gray-400">
          <ShieldIcon />
        </div>

        <!-- URL Input -->
        <form
          @submit.prevent="handleSubmit"
          class="w-full"
        >
          <input
            v-model="urlInput"
            type="text"
            placeholder="Search or enter URL"
            class="w-full h-8 bg-[#272822] text-gray-300 px-8 rounded focus:outline-none focus:ring-1 focus:ring-gray-500 text-sm"
          />
        </form>
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="flex items-center space-x-0 p-0 ml-2">
      <!-- Zoom Controls -->
      <div class="flex space-x-0 mr-2">
        <button
          @click="zoomOut"
          :disabled="zoomLevel <= 50"
          class="mt-0.5 w-7 h-7 rounded-full flex items-center justify-center text-gray-300 hover:bg-[#4A4A4A]"
          :class="{ 'text-gray-600 cursor-not-allowed': zoomLevel <= 50 }"
          title="Zoom Out (Ctrl+-)"
        >
          <MinusIcon />
        </button>

        <button
          @click="resetZoom"
          class="mx-1 w-8 h-8 rounded-full flex items-center justify-center text-gray-300 hover:bg-[#4A4A4A]"
          title="Reset Zoom (Ctrl+0)"
        >
          <span class="text-xs">{{ zoomLevel }}%</span>
        </button>

        <button
          @click="zoomIn"
          :disabled="zoomLevel >= 200"
          class="mt-0.5 w-7 h-7 rounded-full flex items-center justify-center text-gray-300 hover:bg-[#4A4A4A]"
          :class="{ 'text-gray-600 cursor-not-allowed': zoomLevel >= 200 }"
          title="Zoom In (Ctrl+=)"
        >
          <PlusIcon />
        </button>
      </div>

      <!-- History Button -->
      <button
        @click="toggleHistoryModal"
        class="w-8 h-8 rounded flex items-center justify-center text-gray-300 hover:bg-[#4A4A4A]"
        title="History"
      >
        <HistoryIcon />
      </button>

      <!-- Bookmark Button -->
      <button
        @click="handleBookmark"
        class="w-8 h-8 rounded flex items-center justify-center text-gray-300 hover:bg-[#4A4A4A]"
      >
        <BookmarkIcon />
      </button>

      <!-- Extensions Button (placeholder for your browser-like UI) -->
      <button
        class="w-8 h-8 rounded flex items-center justify-center text-gray-300 hover:bg-[#4A4A4A]"
      >
        <DotsIcon />
      </button>

      <!-- User Profile Button -->
      <div class="relative ml-2">
        <button
          @click.stop="toggleProfileDropdown"
          class="profile-button w-8 h-8 rounded-full flex items-center justify-center overflow-hidden border border-gray-600 hover:border-gray-400"
        >
          <!-- Default profile icon or user image -->
          <div class="w-full h-full bg-gray-600 flex items-center justify-center text-white">
            <span class="text-xs">{{ getUserInitial() }}</span>
          </div>
        </button>

        <!-- Profile Dropdown Menu -->
        <div
          v-if="showProfileDropdown"
          class="profile-dropdown absolute right-0 top-10 w-64 bg-[#2A2A2A] border border-gray-700 rounded shadow-lg z-50"
        >
          <!-- User Profile Header -->
          <div class="p-3 flex flex-col items-center border-b border-gray-700">
            <div
              class="w-14 h-14 rounded-full bg-gray-600 flex items-center justify-center text-white mb-2"
            >
              <span class="text-lg">{{ getUserInitial() }}</span>
            </div>
            <div class="text-gray-200 font-medium">{{ userStore.user?.username || 'Guest' }}</div>
            <div class="text-gray-400 text-sm">{{ userStore.user?.email || 'Not logged in' }}</div>
          </div>

          <!-- Dropdown Menu Items -->
          <div class="py-1">
            <button
              class="w-full px-4 py-2 text-left text-gray-300 hover:bg-[#3A3A3A] flex items-center"
            >
              <PasswordIcon class="w-4 h-4 mr-3" />
              Passwords and autofill
            </button>
            <button
              class="w-full px-4 py-2 text-left text-gray-300 hover:bg-[#3A3A3A] flex items-center"
            >
              <UserIcon class="w-4 h-4 mr-3" />
              Manage your Account
            </button>
            <button
              class="w-full px-4 py-2 text-left text-gray-300 hover:bg-[#3A3A3A] flex items-center"
            >
              <EditIcon class="w-4 h-4 mr-3" />
              Customize profile
            </button>
            <button
              class="w-full px-4 py-2 text-left text-gray-300 hover:bg-[#3A3A3A] flex items-center"
            >
              <SyncIcon class="w-4 h-4 mr-3" />
              Sync is on
            </button>
          </div>

          <!-- Other Profiles Section -->
          <div class="border-t border-gray-700 pt-2 pb-1">
            <div class="px-4 py-1 text-sm text-gray-400">Other Chrome profiles</div>
            <button
              class="w-full px-4 py-2 text-left text-gray-300 hover:bg-[#3A3A3A] flex items-center"
            >
              <div
                class="w-5 h-5 rounded-full bg-orange-500 mr-3 flex items-center justify-center text-white"
              >
                <span class="text-xs">O</span>
              </div>
              Oscar
            </button>
            <button
              class="w-full px-4 py-2 text-left text-gray-300 hover:bg-[#3A3A3A] flex items-center"
            >
              <EditIcon class="w-5 h-5 mr-3" />
              Add Chrome profile
            </button>
            <button
              class="w-full px-4 py-2 text-left text-gray-300 hover:bg-[#3A3A3A] flex items-center"
            >
              <UserIcon class="w-5 h-5 mr-3" />
              Open Guest profile
            </button>
            <button
              class="w-full px-4 py-2 text-left text-gray-300 hover:bg-[#3A3A3A] flex items-center"
            >
              <SettingsIcon class="w-5 h-5 mr-3" />
              Manage Chrome profiles
            </button>
            <button
              @click="handleLogout"
              class="w-full px-4 py-2 text-left text-gray-300 hover:bg-[#3A3A3A] flex items-center"
            >
              <LogoutIcon class="w-5 h-5 mr-3" />
              Log out
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- History Modal -->
    <BrowserHistoryModal
      :is-open="showHistoryModal"
      @close="showHistoryModal = false"
      @navigate="handleHistoryNavigation"
    />
  </div>
</template>
