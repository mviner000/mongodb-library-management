<!-- src/views/ExcelCellReference.vue -->
<script setup lang="ts">
  import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue'
  import DeleteDocumentAction from './mongodbtable/DeleteDocumentAction.vue'
  import { getApiBaseUrl } from '@/utils/api'
  import { Dialog, DialogContent, DialogTitle, DialogDescription } from '@/components/ui/dialog'
  import { Button } from '@/components/ui/button'
  import { useToast } from '@/components/ui/toast/use-toast'
  import { AUTH_CONSTANTS } from '@/constants/auth'

  import {
    ChevronDown,
    Circle,
    RotateCcw,
    Archive,
    Globe,
    ArrowDownToLine,
    Trash2,
  } from 'lucide-vue-next'

  import DropdownMenu from './ui/dropdown-menu/DropdownMenu.vue'
  import DropdownMenuTrigger from './ui/dropdown-menu/DropdownMenuTrigger.vue'
  import DropdownMenuContent from './ui/dropdown-menu/DropdownMenuContent.vue'
  import DropdownMenuItem from './ui/dropdown-menu/DropdownMenuItem.vue'

  const API_BASE = getApiBaseUrl()
  const { toast } = useToast()

  const props = defineProps<{
    selectedCell: { colIndex: number; rowNumber: number } | null
    selectedRows: Set<string>
    collectionName: string
    documents: any[]
    currentPage: number
    pageSize: number
    isSidebarOpen: boolean
  }>()

  const emit = defineEmits<{
    (e: 'document-deleted'): void
    (e: 'delete-start', id: string): void
    (e: 'delete-end'): void
    (e: 'reset-selection'): void
    (e: 'toggle-view'): void
    (e: 'view-change', view: string): void
  }>()

  // Reference to the DeleteDocumentAction component
  const deleteDocumentRef = ref<InstanceType<typeof DeleteDocumentAction> | null>(null)

  // Batch delete dialog state
  const showBatchDeleteDialog = ref(false)
  const isBatchDeleting = ref(false)
  const batchConfirmationText = ref('')
  const confirmationRequired = 'confirm-delete'
  const batchInputRef = ref<HTMLInputElement | null>(null)

  // Menu state - this will control both dropdown and tab bar
  const selected = ref('Default')

  // Define the options for the view dropdown and tabs
  const options = [
    { label: 'Default' },
    { label: 'Recoveries' },
    { label: 'Archives' },
    { label: 'All' },
  ]

  // Helper function to get the appropriate icon component based on menu item
  const getIconComponent = (label: string) => {
    switch (label) {
      case 'Default':
        return Circle
      case 'Recoveries':
        return RotateCcw
      case 'Archives':
        return Archive
      case 'All':
        return Globe
      default:
        return Circle
    }
  }

  // Computed property to get the icon component for the currently selected label
  const selectedIconComponent = computed(() => {
    return getIconComponent(selected.value)
  })

  // Handle selecting an option from the dropdown or tab bar
  const select = (label: string) => {
    selected.value = label

    // Map selection to appropriate view type value
    let viewType = ''
    if (label === 'Archives') {
      viewType = 'archives'
    } else if (label === 'Recoveries') {
      viewType = 'recoveries'
    } else if (label === 'Default') {
      viewType = 'empty-or-recovered'
    } else if (label === 'All') {
      viewType = 'all'
    }

    // Emit the view-change event with the view type and reset the selection in the parent component
    emit('view-change', viewType)
    emit('reset-selection')
  }

  // Add event listener for batch key handling
  onMounted(() => {
    document.addEventListener('keydown', handleBatchKeyDown)
  })

  onBeforeUnmount(() => {
    document.removeEventListener('keydown', handleBatchKeyDown)
  })

  // Open batch delete dialog
  const openBatchDeleteDialog = () => {
    emit('delete-start', 'batch')
    batchConfirmationText.value = ''
    showBatchDeleteDialog.value = true
    // Focus the input after dialog is open
    setTimeout(() => {
      batchInputRef.value?.focus()
    }, 100)
  }

  // Close batch delete dialog
  const closeBatchDeleteDialog = () => {
    showBatchDeleteDialog.value = false
    batchConfirmationText.value = ''
    // Only emit delete-end if the dialog was closed manually,
    // not after a successful/failed delete (handled in confirmBatchDelete)
    if (!isBatchDeleting.value) {
      emit('delete-end')
    }
  }

  // Add batch delete handler
  const confirmBatchDelete = async () => {
    if (batchConfirmationText.value !== confirmationRequired || isBatchDeleting.value) return

    isBatchDeleting.value = true
    try {
      // Show processing notification
      const response = await fetch(
        `${API_BASE}/collections/${props.collectionName}/documents/batch-delete`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${localStorage.getItem(AUTH_CONSTANTS.TOKEN_KEY)}`,
          },
          body: JSON.stringify({ ids: [...props.selectedRows] }),
        }
      )

      const { success, error } = await response.json()

      if (success) {
        toast({
          title: 'Documents deleted',
          description: `${props.selectedRows.size} documents were successfully removed`,
          variant: 'success',
        })
        emit('document-deleted')
        emit('reset-selection')
        showBatchDeleteDialog.value = false // Close dialog on success
      } else {
        toast({
          title: 'Batch delete failed',
          description: error || 'Failed to delete documents',
          variant: 'destructive',
        })
      }
    } catch (error) {
      toast({
        title: 'Error deleting documents',
        description: String(error),
        variant: 'destructive',
      })
    } finally {
      isBatchDeleting.value = false
      // If the dialog is still open (e.g., on failure), keep it open until user closes,
      // but if it was successful, it's already closed above.
      emit('delete-end')
      // Ensure confirmation text is cleared if delete fails and dialog stays open
      if (showBatchDeleteDialog.value) {
        batchConfirmationText.value = ''
      }
    }
  }

  // Handle Enter key press for batch dialog
  const handleBatchKeyDown = (event: KeyboardEvent) => {
    if (
      event.key === 'Enter' &&
      showBatchDeleteDialog.value &&
      batchConfirmationText.value === confirmationRequired &&
      !isBatchDeleting.value
    ) {
      event.preventDefault()
      confirmBatchDelete()
    }
  }

  // Store information about document to delete
  const documentToDelete = ref<{ id: string; rowNumber: number } | null>(null)
  const isDeleting = ref(false)

  // Debug watcher for props
  watch(
    () => props.selectedRows,
    (newVal) => {
      // Removed console.log
    },
    { immediate: true }
  )

  watch(
    () => props.collectionName,
    (newVal) => {
      // Removed console.log
    },
    { immediate: true }
  )

  const getColumnLabel = (index: number): string => {
    let label = ''
    let i = index
    do {
      const remainder = i % 26
      label = String.fromCharCode(65 + remainder) + label
      i = Math.floor(i / 26) - 1
    } while (i >= 0)
    return label
  }

  const cellReference = computed(() => {
    if (!props.selectedCell) return 'A1' // Default when no cell is selected
    const { colIndex, rowNumber } = props.selectedCell
    return `${getColumnLabel(colIndex)}${rowNumber}`
  })

  // Get the document ID from the selected row
  const getSelectedDocumentId = (): string | null => {
    if (props.selectedRows.size !== 1) return null

    // Get the first (and only) item from the Set
    const [documentId] = [...props.selectedRows]
    return documentId
  }

  // Calculate the row number based on the selected document ID
  const getRowNumberForDocument = (documentId: string): number => {
    // Find the index of the document in the props.documents array
    const index = props.documents.findIndex((doc) => doc._id.$oid === documentId)

    if (index === -1) return 0

    // Calculate row number based on pagination
    const rowNumber = (props.currentPage - 1) * props.pageSize + index + 1

    return rowNumber
  }

  // Handle click on delete button
  const handleDeleteClick = () => {
    const documentId = getSelectedDocumentId()

    if (!documentId) {
      return
    }

    const rowNumber = getRowNumberForDocument(documentId)

    documentToDelete.value = { id: documentId, rowNumber }

    // Use a small delay to ensure component is updated
    setTimeout(() => {
      deleteDocumentRef.value?.openDeleteDialog()
    }, 50)
  }

  // Archive handler
  const handleArchiveClick = async () => {
    const documentId = getSelectedDocumentId()

    if (!documentId) {
      toast({ title: 'Error', description: 'No document selected' })
      return
    }

    try {
      const response = await fetch(
        `${API_BASE}/collections/${props.collectionName}/documents/${documentId}/archive`,
        {
          method: 'PUT',
          headers: {
            Authorization: `Bearer ${localStorage.getItem(AUTH_CONSTANTS.TOKEN_KEY)}`,
          },
        }
      )

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || 'Archive failed')
      }

      toast({
        title: 'Document archived',
        description: 'The document has been archived successfully',
        variant: 'success',
      })
      emit('document-deleted')
      emit('reset-selection')
    } catch (error: unknown) {
      let message = 'Failed to archive document'

      if (error instanceof Error) {
        message = error.message
      }

      toast({
        title: 'Archive error',
        description: message,
        variant: 'destructive',
      })
    }
  }

  // batch archive handler
  const handleBatchArchive = async () => {
    if (props.selectedRows.size === 0) {
      toast({ title: 'No selection', description: 'Please select items to archive' })
      return
    }

    try {
      const response = await fetch(
        `${API_BASE}/collections/${props.collectionName}/documents/batch-archive`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${localStorage.getItem(AUTH_CONSTANTS.TOKEN_KEY)}`,
          },
          body: JSON.stringify({ ids: [...props.selectedRows] }),
        }
      )

      const data = await response.json()

      if (data.success) {
        toast({
          title: 'Archived successfully',
          description: data.data.message,
          variant: 'success',
        })
        emit('document-deleted')
        emit('reset-selection')
      } else {
        toast({
          title: 'Archive failed',
          description: data.error || 'Failed to archive documents',
          variant: 'destructive',
        })
      }
    } catch (error) {
      toast({
        title: 'Archive error',
        description: String(error),
        variant: 'destructive',
      })
    }
  }

  // Single recover handler
  const handleRecoverClick = async () => {
    const documentId = getSelectedDocumentId()
    if (!documentId) {
      toast({ title: 'Error', description: 'No document selected' })
      return
    }

    try {
      const response = await fetch(
        `${API_BASE}/collections/${props.collectionName}/documents/${documentId}/recover`,
        {
          method: 'PUT',
          headers: {
            Authorization: `Bearer ${localStorage.getItem(AUTH_CONSTANTS.TOKEN_KEY)}`,
          },
        }
      )

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || 'Recover failed')
      }

      toast({
        title: 'Document recovered',
        description: 'The document has been recovered successfully',
        variant: 'success',
      })
      emit('document-deleted')
      emit('reset-selection')
    } catch (error: unknown) {
      toast({
        title: 'Recover error',
        description: error instanceof Error ? error.message : 'Failed to recover document',
        variant: 'destructive',
      })
    }
  }

  // Batch recover handler
  const handleBatchRecover = async () => {
    if (props.selectedRows.size === 0) {
      toast({ title: 'No selection', description: 'Please select items to recover' })
      return
    }

    try {
      const response = await fetch(
        `${API_BASE}/collections/${props.collectionName}/documents/batch-recover`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${localStorage.getItem(AUTH_CONSTANTS.TOKEN_KEY)}`,
          },
          body: JSON.stringify({ ids: [...props.selectedRows] }),
        }
      )

      const data = await response.json()

      if (data.success) {
        toast({
          title: 'Recovered successfully',
          description: data.data.message,
          variant: 'success',
        })
        emit('document-deleted')
        emit('reset-selection')
      } else {
        toast({
          title: 'Recover failed',
          description: data.error || 'Failed to recover documents',
          variant: 'destructive',
        })
      }
    } catch (error) {
      toast({
        title: 'Recover error',
        description: String(error),
        variant: 'destructive',
      })
    }
  }

  // Event handlers for delete operations
  const onDocumentDeleted = () => {
    documentToDelete.value = null
    isDeleting.value = false
    emit('document-deleted')
    emit('reset-selection') // Also reset selection after single document deletion
  }

  // Removed debug log when the component mounts
</script>

<template>
  <!-- ExcelCellReference main div -->
  <div
    class="fixed h-[42px] z-30 top-14 w-screen flex items-center bg-white border-b border-b-gray-400 transition-all duration-300 ease-in-out"
    :class="isSidebarOpen ? 'left-[280px]' : 'left-0'"
  >
    <!-- Cell reference box (e.g., A1) -->
    <div class="flex items-center px-2">
      <div class="flex items-center cursor-pointer">
        <span class="text-sm font-bold text-gray-700">{{ cellReference }}</span>
      </div>
    </div>

    <!-- fx formula indicator -->
    <div class="flex items-center px-3 text-gray-500">
      <span class="text-sm italic">fx</span>
    </div>

    <!-- Empty space -->
    <div class="flex-1 h-full"></div>
    <div
      class="flex transition-all duration-300 ease-in-out"
      :class="isSidebarOpen ? 'mr-[280px]' : 'mr-[0px]'"
    >
      <!-- single selection buttons -->
      <div
        v-if="selectedRows.size === 1"
        class="mr-5 flex gap-2"
      >
        <!-- Single Recover Button -->
        <button
          v-if="selected === 'Archives' || selected === 'All'"
          @click="handleRecoverClick"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-green-100 text-green-600 border-green-300 hover:bg-green-200"
        >
          <ArrowDownToLine class="h-3 w-3 mr-1" />
          Recover 1 Item
        </button>

        <!-- Single Archive Button -->
        <button
          v-if="selected === 'Default' || selected === 'Recoveries' || selected === 'All'"
          @click="handleArchiveClick"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-blue-100 text-blue-500 border-blue-300 hover:bg-blue-200"
        >
          <Archive class="h-3 w-3 mr-1" />
          Archive 1 Item
        </button>

        <!-- Single Delete Button -->
        <button
          v-if="selected === 'Archives' || selected === 'All'"
          @click="handleDeleteClick"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-red-100 text-red-500 border-red-300 hover:bg-red-200"
        >
          <Trash2 class="h-3 w-3 mr-1" />
          Delete 1 Item
        </button>
      </div>

      <!-- buttons for multiple selections -->
      <div
        v-if="selectedRows.size > 1"
        class="mr-5 flex gap-2"
      >
        <!-- Batch Recovery Button -->
        <button
          v-if="selected === 'Archives' || selected === 'All'"
          @click="handleBatchRecover"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-green-100 text-green-600 border-green-300 hover:bg-green-200"
        >
          <ArrowDownToLine class="h-3 w-3 mr-1" />
          Batch Recover {{ selectedRows.size }} Items
        </button>
        <!-- Batch Archive Button -->
        <button
          v-if="selected === 'Default' || selected === 'Recoveries' || selected === 'All'"
          @click="handleBatchArchive"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-blue-100 text-blue-500 border-blue-300 hover:bg-blue-200"
        >
          <Archive class="h-3 w-3 mr-1" />
          Batch Archive {{ selectedRows.size }} Items
        </button>

        <!-- Batch Delete Button -->
        <button
          v-if="selected === 'Archives' || selected === 'All'"
          @click="openBatchDeleteDialog"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-red-100 text-red-500 border-red-300 hover:bg-red-200"
        >
          <Trash2 class="h-3 w-3 mr-1" />
          Batch Delete {{ selectedRows.size }} Items
        </button>
      </div>

      <!-- Empty space -->

      <!-- options for view as dropdown with shadcn menubar -->
      <div class="pl-3 border-l border-gray-400 pr-4">
        <DropdownMenu>
          <DropdownMenuTrigger
            class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-gray-100 text-gray-700 border-gray-300 hover:bg-gray-200 gap-1"
          >
            <span class="inline-flex items-center"
              ><component
                :is="selectedIconComponent"
                class="h-3 w-3 mr-1"
              />{{ selected }}</span
            >
            <ChevronDown class="h-3 w-3" />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem
              v-for="item in options"
              :key="item.label"
              @click="select(item.label)"
              class="flex items-center gap-2 text-sm"
            >
              <component
                :is="getIconComponent(item.label)"
                class="h-3 w-3"
              />
              {{ item.label }}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>

    <!-- Single Delete document dialog -->
    <DeleteDocumentAction
      v-if="documentToDelete"
      :collection-name="collectionName"
      :document-id="documentToDelete.id"
      :row-number="documentToDelete.rowNumber"
      ref="deleteDocumentRef"
      @deleted="onDocumentDeleted"
      @delete-start="(id) => $emit('delete-start', id)"
      @delete-end="$emit('delete-end')"
    />

    <!-- Batch Delete confirmation dialog -->
    <Dialog
      :open="showBatchDeleteDialog"
      @update:open="(val) => val === false && closeBatchDeleteDialog()"
    >
      <DialogContent
        class="custom-delete-dialog p-0 overflow-hidden border-rose-200"
        @keydown.enter.prevent="
          batchConfirmationText === confirmationRequired && !isBatchDeleting && confirmBatchDelete()
        "
      >
        <DialogTitle class="sr-only">Batch Delete Confirmation</DialogTitle>
        <DialogDescription class="sr-only">
          Please confirm your intention to delete {{ selectedRows.size }} documents by typing the
          confirmation text.
        </DialogDescription>

        <div class="bg-rose-100 text-rose-700 p-4 border-b border-rose-200 flex items-center">
          <div class="flex-1">You are about to delete {{ selectedRows.size }} documents</div>
        </div>

        <div class="p-4 bg-white">
          <p class="text-sm text-gray-700 mb-3">
            To confirm, type
            <span class="font-medium text-rose-600">{{ confirmationRequired }}</span> in the box
            below
          </p>
          <input
            v-model="batchConfirmationText"
            placeholder="Type confirmation text"
            ref="batchInputRef"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rose-500"
            aria-label="Confirmation text"
          />
          <p class="text-xs text-gray-500 mt-2">Press Enter to confirm when text is correct</p>
        </div>

        <div class="flex justify-end p-3 pt-0 bg-white">
          <Button
            @click="confirmBatchDelete"
            size="lg"
            class="w-full bg-rose-600 hover:bg-rose-700 text-white"
            :disabled="batchConfirmationText !== confirmationRequired || isBatchDeleting"
          >
            {{ isBatchDeleting ? 'Deleting...' : `Delete ${selectedRows.size} documents` }}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  </div>
  <div class="sheets-tabs-container">
    <div class="sheets-tabs-wrapper">
      <button class="nav-button">
        <span class="nav-icon">◀</span>
      </button>

      <div class="sheets-tabs">
        <div
          v-for="item in options"
          :key="item.label"
          :class="['sheet-tab', { active: selected === item.label }]"
          @click="select(item.label)"
        >
          <span class="sheet-name">{{ item.label }}</span>
          <div class="sheet-dropdown">
            <span class="dropdown-icon">▼</span>
          </div>
        </div>
      </div>

      <button class="nav-button add-sheet">
        <span class="add-icon">+</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
  .custom-delete-dialog {
    max-width: 500px;
    border-radius: 4px;
  }

  /* Keep button styles within the dialog consistent */
  .bg-rose-100 {
    background-color: #ffebee;
  }
  .text-rose-700 {
    color: #c6282d;
  }
  .border-rose-200 {
    border-color: #ffcdd2;
  }
  .text-rose-600 {
    color: #e53935;
  }
  .bg-rose-600 {
    background-color: #e53935;
  }
  .hover\:bg-rose-700:hover {
    background-color: #d32f2f;
  }

  /* for footer tab bar */
  .sheets-tabs-container {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    background-color: #f1f3f4;
    border-top: 1px solid #dadce0;
    z-index: 1000;
  }

  .sheets-tabs-wrapper {
    display: flex;
    align-items: center;
    height: 36px;
    padding: 0 4px;
  }

  .sheets-tabs {
    display: flex;
    overflow-x: auto;
    scrollbar-width: none; /* Hide scrollbar for Firefox */
    -ms-overflow-style: none; /* Hide scrollbar for IE and Edge */
  }

  .sheets-tabs::-webkit-scrollbar {
    display: none; /* Hide scrollbar for Chrome, Safari and Opera */
  }

  .sheet-tab {
    display: flex;
    align-items: center;
    min-width: 60px;
    max-width: 180px;
    height: 28px;
    margin: 4px 2px;
    padding: 0 8px;
    border-radius: 4px 4px 0 0;
    background-color: #e0e0e0;
    color: #5f6368;
    font-size: 13px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition:
      background-color 0.2s,
      color 0.2s,
      border-bottom 0.2s;
  }

  .sheet-tab.active {
    background-color: white;
    color: #1a73e8;
    border-bottom: 2px solid #1a73e8;
  }

  .sheet-name {
    flex-grow: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sheet-dropdown {
    position: relative;
    margin-left: 4px;
    padding: 2px;
  }

  .dropdown-icon {
    font-size: 8px;
    color: #5f6368;
  }

  .nav-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    margin: 0 2px;
    border: none;
    border-radius: 50%;
    background-color: transparent;
    cursor: pointer;
  }

  .nav-button:hover {
    background-color: #e0e0e0;
  }

  .add-sheet {
    color: #5f6368;
  }

  .add-icon {
    font-size: 18px;
  }

  .nav-icon {
    font-size: 12px;
    color: #5f6368;
  }
</style>
