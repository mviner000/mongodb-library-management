<template>
  <div class="sidebar-container">
    <transition name="slide">
      <div
        v-if="isOpen"
        class="sidebar"
      >
        <div class="sidebar-content">
          <h2 class="sidebar-title">📌 Pinned Items</h2>
          <ScrollArea class="h-[calc(100vh-100px)]">
            <div class="pr-4">
              <div
                v-for="doc in sortedPinnedDocuments"
                :key="doc._id.$oid"
                class="group flex items-center gap-2 p-2 mb-1 rounded-md hover:bg-gray-100 cursor-pointer transition-colors pinned-item"
                @click="navigateToDocument(doc._id.$oid)"
              >
                <span class="text-lg">📌</span>
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium truncate">{{ doc.label || doc._id.$oid }}</div>
                  <div class="text-xs text-gray-500 truncate">
                    pinned {{ formatRelativeTime(getPinnedTime(doc)) }}
                  </div>
                </div>
              </div>
              <div
                v-if="pinnedDocuments.length === 0"
                class="text-sm text-gray-500 italic p-2"
              >
                No pinned items
              </div>
            </div>
          </ScrollArea>
        </div>
      </div>
    </transition>

    <!-- Half-circle trigger button - Excel style -->
    <div
      class="trigger-button opacity-50 hover:opacity-100"
      :class="{ open: isOpen }"
      @click="toggleSidebar"
    >
      <div class="icon-container">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line
            x1="3"
            y1="12"
            x2="21"
            y2="12"
          ></line>
          <line
            v-if="!isOpen"
            x1="3"
            y1="6"
            x2="21"
            y2="6"
          ></line>
          <line
            v-if="!isOpen"
            x1="3"
            y1="18"
            x2="21"
            y2="18"
          ></line>
          <line
            v-if="isOpen"
            x1="18"
            y1="6"
            x2="6"
            y2="18"
          ></line>
          <line
            v-if="isOpen"
            x1="6"
            y1="6"
            x2="18"
            y2="18"
          ></line>
        </svg>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ScrollArea } from '@/components/ui/scroll-area'
  import { formatDistanceToNow, parseISO, isValid } from 'date-fns'
  import { computed } from 'vue'

  // Define Document interface to match store's Document interface
  interface DocumentId {
    $oid: string
  }

  interface PinnedHistoryItem {
    action: string
    user_id: string
    timestamp:
      | {
          $date?: {
            $numberLong: string
          }
        }
      | string
  }

  interface Document {
    _id: DocumentId
    label?: string
    updated_at?: string
    pinned_history?: PinnedHistoryItem[]
    [key: string]: any
  }

  const props = defineProps({
    isOpen: {
      type: Boolean,
      default: true,
    },
    pinnedDocuments: {
      type: Array as () => Document[],
      default: () => [],
    },
  })

  const emit = defineEmits(['toggle', 'navigate'])

  // Get the most recent pin timestamp from pinned_history if available, otherwise use updated_at
  const getPinnedTime = (doc: Document): string => {
    // Check if document has pinned_history
    if (doc.pinned_history && doc.pinned_history.length > 0) {
      // Find the most recent pin action
      const pinActions = doc.pinned_history.filter((item) => item.action === 'pin')
      if (pinActions.length > 0) {
        // Sort by timestamp descending and get the most recent
        const sortedPins = [...pinActions].sort((a, b) => {
          const getTimestamp = (item: PinnedHistoryItem) => {
            if (typeof item.timestamp === 'string') {
              return new Date(item.timestamp).getTime()
            } else if (item.timestamp && item.timestamp.$date && item.timestamp.$date.$numberLong) {
              return parseInt(item.timestamp.$date.$numberLong)
            }
            return 0
          }

          return getTimestamp(b) - getTimestamp(a)
        })

        const mostRecentPin = sortedPins[0]
        if (typeof mostRecentPin.timestamp === 'string') {
          return mostRecentPin.timestamp
        } else if (
          mostRecentPin.timestamp &&
          mostRecentPin.timestamp.$date &&
          mostRecentPin.timestamp.$date.$numberLong
        ) {
          // Convert milliseconds to ISO date string
          return new Date(parseInt(mostRecentPin.timestamp.$date.$numberLong)).toISOString()
        }
      }
    }

    // Fallback to updated_at if no pinned_history or unable to parse
    return doc.updated_at || ''
  }

  // Sort documents by most recently pinned time
  const sortedPinnedDocuments = computed(() => {
    return [...props.pinnedDocuments].sort((a, b) => {
      const timeA = getPinnedTime(a)
      const timeB = getPinnedTime(b)

      const dateA = timeA ? new Date(timeA).getTime() : 0
      const dateB = timeB ? new Date(timeB).getTime() : 0

      return dateB - dateA // Most recent first
    })
  })

  const toggleSidebar = () => {
    emit('toggle')
  }

  // Function for relative time without "ago" since we'll add "pinned" before it
  const formatRelativeTime = (dateString: string) => {
    if (!dateString) return ''

    try {
      const date = parseISO(dateString)

      if (!isValid(date)) {
        return 'at unknown time'
      }

      // Use the browser's local time for comparison, which should match the user's timezone
      const timeAgo = formatDistanceToNow(date, { addSuffix: false })

      return timeAgo.toLowerCase() + ' ago'
    } catch (error) {
      console.error('Error formatting date:', error, dateString)
      return 'at unknown time'
    }
  }

  const navigateToDocument = (docId: string) => {
    emit('navigate', docId)
  }
</script>

<style scoped>
  .sidebar-container {
    position: relative;
    z-index: 40;
  }

  .sidebar {
    position: fixed;
    top: 0;
    left: 0;
    height: 100vh;
    width: 280px;
    background-color: #f8f9fa;
    border-right: 1px solid #d4d4d8;
    z-index: 40;
    margin-top: 0px; /* Adjust based on your Excel header height */
    @apply shadow-lg;
  }

  .sidebar-content {
    padding: 16px;
  }

  .sidebar-title {
    font-size: 14px;
    font-weight: 500;
    color: #4b5563;
    margin-bottom: 16px;
  }

  .sidebar-menu {
    display: flex;
    flex-direction: column;
  }

  .trigger-button {
    position: fixed;
    top: 75%;
    right: auto;
    left: 0;
    transform: translateY(-50%);
    width: 22px;
    height: 60px;
    background-color: #217346; /* Excel green color */
    border-radius: 0 4px 4px 0;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      all 0.3s ease,
      opacity 0.2s ease;
    z-index: 41;
    box-shadow: 1px 1px 3px rgba(0, 0, 0, 0.1);
  }

  .trigger-button.open {
    left: 280px;
    background-color: #185a34; /* Darker green when open */
  }

  .icon-container {
    display: flex;
    justify-content: center;
    align-items: center;
    color: white;
    margin-left: -2px;
  }

  /* Animation */
  .slide-enter-active,
  .slide-leave-active {
    transition: transform 0.3s ease;
  }

  .slide-enter-from,
  .slide-leave-to {
    transform: translateX(-100%);
  }

  .pinned-item {
    @apply transition-colors duration-200 ease-in-out;
  }

  .pinned-item:hover {
    @apply bg-blue-50;
  }

  .pinned-item:active {
    @apply bg-blue-100;
  }
</style>
