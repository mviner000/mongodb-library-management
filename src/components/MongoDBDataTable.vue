<!-- src/components/MongoDBDataTable.vue -->
<script setup lang="ts">
  // --- Script Section (Pinia Integrated) ---
  import { ref, computed, watch, onMounted, Ref, inject, nextTick, onBeforeUnmount } from 'vue'
  import { useRoute, useRouter } from 'vue-router'
  import { storeToRefs } from 'pinia'
  import { useDataTableStore } from '@/store/dataTableStore' // Make sure this path is correct
  import { useDebounceFn } from '@vueuse/core'

  // Import UI Components as used in your *original* template
  import { Cross2Icon, ReloadIcon, PlusCircledIcon } from '@radix-icons/vue' // [cite: 1]
  import { Button } from '@/components/ui/button' // [cite: 1]
  import { Input } from '@/components/ui/input' // [cite: 41]
  import { TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table' // Used implicitly by <Table...> tags in old template
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select' // [cite: 39, 82]
  import {
    Pagination,
    PaginationList,
    PaginationListItem,
    PaginationFirst,
    PaginationLast,
    PaginationNext,
    PaginationPrev,
  } from '@/components/ui/pagination' // [cite: 79]

  import { ScrollArea } from '@/components/ui/scroll-area' // [cite: 40]
  // Remove toast import if not used directly in setup, it's in the store
  // import { useToast } from '@/components/ui/toast/use-toast';
  import ExcelCellReference from './ExcelCellReference.vue' // [cite: 2]
  import TableActions from './mongodbtable/TableActions.vue' // [cite: 57]
  import StickyTableActions from './mongodbtable/StickyTableActions.vue' // [cite: 59]
  import MongoDBDataTableNavbar from './MongoDBDataTableNavbar.vue' // [cite: 1]
  import StickyLeftSidebar from './StickyLeftSidebar.vue'
  import { useUserStore } from '@/store/useUserStore'
  // Remove FooterTabsBar import if not used in old template
  // import FooterTabsBar from './FooterTabsBar.vue';

  // Store setup
  const dataTableStore = useDataTableStore()
  // Destructure state and getters into reactive refs
  const {
    collectionName,
    documents,
    collectionSchema,
    isLoading,
    errorMessage,
    pageSize,
    currentPage,
    newDocument,
    isAdding,
    editingCell,
    editValue,
    isSaving,
    selectedRows,
    currentView,
    pendingDeleteId,
    referenceOptions,
    loadingReferences,
    errorColumn,
    addingRowError,
    totalPages,
    paginatedDocuments,
    tableHeaders,
    columnWidths,
    allSelected,
  } = storeToRefs(dataTableStore)

  // Destructure actions (they are just functions)
  const {
    fetchCollections,
    setCollection,
    fetchDocuments,
    fetchReferenceOptions,
    getReferenceLabel,
    startAdding,
    cancelAdding,
    saveNewDocument,
    startEditingCell,
    cancelEdit,
    saveEdit,
    toggleRow,
    resetSelection,
    changeView,
    setPage, // Renamed from onPageChange
    setPageSize, // Added action
    updateColumnWidth,
    resetColumnWidth,
    saveColumnWidthsToBackend,
    clearError, // Renamed from closeError
    getSchemaInfo,
    isReferenceField,
    getReferencedCollection,
    // fetchSchema, // Optional direct access
  } = dataTableStore

  const userStore = useUserStore()
  const { user } = storeToRefs(userStore)

  // Local component state
  const route = useRoute()
  const router = useRouter()
  const isSplit = inject<Ref<boolean>>('isSplit')!
  const scrollContainer = ref<HTMLElement | null>(null) // [cite: 2]
  // const filterQuery = ref('{}'); // Kept local filter state

  const isSidebarOpen = ref(false)
  const searchQuery = ref<Record<string, string>>({}) // [cite: 41]
  const resizingState = ref({
    isResizing: false,
    header: '',
    startX: 0,
    startWidth: 0,
    currentWidth: 0,
  }) // [cite: 20]
  const alphaResizingState = ref({
    isResizing: false,
    columnIndex: -1,
    startX: 0,
    startWidth: 0,
    currentWidth: 0,
  }) // [cite: 9]
  const selectedCell = ref<{ colIndex: number; rowNumber: number } | null>(null) // [cite: 2]
  const timeoutId = ref<number | null>(null) // For error message timeout [cite: 1] - Keep or remove based on preference
  const showPinTooltip = ref(false)
  // Props
  const props = defineProps<{
    selectedCollection?: string
    name?: string
  }>()

  // --- Computed properties specific to component ---
  const getColumnLabel = (index: number): string => {
    let label = ''
    let i = index
    do {
      const remainder = i % 26
      label = String.fromCharCode(65 + remainder) + label
      i = Math.floor(i / 26) - 1
    } while (i >= 0)
    return label
  } // [cite: 9]

  const columnLetters = computed(() => {
    return tableHeaders.value.map((_, index) => getColumnLabel(index))
  }) // [cite: 9]

  const numberColumnWidth = computed(() => {
    const maxDigits = documents.value.length > 0 ? String(documents.value.length).length : 1
    return `${Math.max(3, maxDigits + 1)}ch`
  }) // [cite: 18] adjusted logic slightly

  const totalTableWidth = computed(() => {
    const dataColumnsWidth = tableHeaders.value.reduce(
      (acc, header) => acc + (columnWidths.value[header] || 200),
      0
    )
    const selectColWidth = 40 // [cite: 5]
    const rowNumColWidth = 30 // As per original template [cite: 7] - Adjust if needed
    const actionsColWidth = 60 // As per original styles [cite: 125]

    return selectColWidth + rowNumColWidth + dataColumnsWidth + actionsColWidth + 1
  }) // Adjusted calculation based on old template fixed widths

  // --- Utility Functions ---
  const isFieldRequired = (field: string): boolean => {
    return collectionSchema.value.required?.includes(field) || false
  } // [cite: 22]

  const formatSchemaValue = (value: any, bsonType?: string | string[]) => {
    if (value === undefined || value === null) return ''
    const type = bsonType ? (Array.isArray(bsonType) ? bsonType[0] : bsonType) : typeof value

    if (type === 'date' && value instanceof Date) {
      // Check if it's already a Date object
      return value.toLocaleString()
    } else if (type === 'date') {
      // Try parsing if it's not a Date object
      try {
        return new Date(value).toLocaleString()
      } catch {
        return String(value) // Fallback
      }
    }
    if (typeof value === 'object') {
      return JSON.stringify(value, null, 2) // Keep original formatting [cite: 57]
    }
    return String(value)
  } // [cite: 56, 57]

  const filteredOptions = (field: string) => {
    const refCollection = getReferencedCollection(field)
    if (!refCollection) return []
    const options = referenceOptions.value[refCollection] || []
    const query = (searchQuery.value[field] || '').toLowerCase()
    return options.filter((opt) => opt.label.toLowerCase().includes(query))
  } // [cite: 42]

  // --- Watchers ---

  // Watch route parameter 'name'
  watch(
    () => route.params.name,
    (newName) => {
      const nameStr = Array.isArray(newName) ? newName[0] : newName
      if (nameStr && nameStr !== collectionName.value) {
        setCollection(nameStr)
      }
    },
    { immediate: true }
  )

  // Watch the prop 'selectedCollection'
  watch(
    () => props.selectedCollection,
    (newVal) => {
      if (newVal && newVal !== collectionName.value) {
        setCollection(newVal)
        if (route.params.name !== newVal) {
          router.push(`/collection/${newVal}`)
        }
      }
    }
  )

  // Watch the store's collectionName to sync the route if needed
  watch(collectionName, (newName, oldName) => {
    if (newName && newName !== oldName && route.params.name !== newName) {
      router.push(`/collection/${newName}`)
    }
  })

  // Watch for error message to auto-clear (optional, kept from original logic)
  watch(errorMessage, (newVal) => {
    if (newVal) {
      if (timeoutId.value) {
        clearTimeout(timeoutId.value)
      }
      timeoutId.value = setTimeout(() => {
        clearError() // Use store action
        timeoutId.value = null
      }, 2500) as unknown as number // [cite: 1] matching timeout
    }
  })

  // Watch isAdding to prefetch reference options (kept from original logic)
  watch(isAdding, async (newVal) => {
    if (newVal) {
      tableHeaders.value.forEach((field) => {
        // Use tableHeaders getter
        const refCollection = getReferencedCollection(field)
        if (refCollection && !referenceOptions.value[refCollection]) {
          fetchReferenceOptions(refCollection) // Use store action
        }
      })
    }
  }) // [cite: 67] reference fetching logic

  // --- Lifecycle Hooks ---
  onMounted(async () => {
    console.log('MongoDBDataTable mounted (Pinia + Old Style).')

    // Add user ID logging
    if (user.value) {
      console.log('User ID:', user.value.id)
    } else {
      console.log('User not authenticated')
    }

    await fetchCollections()
    const routeName = Array.isArray(route.params.name) ? route.params.name[0] : route.params.name
    const initialCollection = routeName || props.selectedCollection

    if (
      initialCollection &&
      typeof initialCollection === 'string' &&
      initialCollection !== collectionName.value
    ) {
      await setCollection(initialCollection)
    } else if (collectionName.value && documents.value.length === 0 && !isLoading.value) {
      try {
        await dataTableStore.fetchSchema()
        await dataTableStore.fetchDocuments()
      } catch (error) {
        console.error('Error fetching initial schema/documents in onMounted:', error)
      }
    }
  })

  // --- Methods ---

  // Error closing (if keeping the manual close button)
  const closeErrorManual = () => {
    if (timeoutId.value) {
      clearTimeout(timeoutId.value)
      timeoutId.value = null
    }
    clearError() // Use store action
  } // Matches original @click="closeError" [cite: 1]

  // Debounced function for saving widths
  const debouncedSaveWidths = useDebounceFn(async () => {
    await saveColumnWidthsToBackend() // Call store action
  }, 750) // Keep debounce consistent

  // --- Column Resizing Handlers ---
  // These call store actions now but retain the local state for visual feedback during drag

  const startAlphaResize = (columnIndex: number, event: MouseEvent) => {
    const header = tableHeaders.value[columnIndex] // Use getter
    if (!header) return
    const currentWidth = columnWidths.value[header] || 200 // Use getter

    alphaResizingState.value = {
      isResizing: true,
      columnIndex,
      startX: event.clientX,
      startWidth: currentWidth,
      currentWidth: currentWidth,
    }
    document.addEventListener('mousemove', handleAlphaMouseMove)
    document.addEventListener('mouseup', stopAlphaResize)
    event.preventDefault()
  } // [cite: 14]

  const handleAlphaMouseMove = (event: MouseEvent) => {
    if (!alphaResizingState.value.isResizing) return
    const delta = event.clientX - alphaResizingState.value.startX
    const newWidth = Math.max(50, alphaResizingState.value.startWidth + delta)
    alphaResizingState.value.currentWidth = newWidth

    const header = tableHeaders.value[alphaResizingState.value.columnIndex]
    if (header && collectionSchema.value.ui) {
      // Temporarily update local schema for visual feedback
      collectionSchema.value.ui.columnWidths = {
        ...collectionSchema.value.ui.columnWidths,
        [header]: newWidth,
      }
    }
  }

  const stopAlphaResize = async () => {
    if (!alphaResizingState.value.isResizing) return
    const header = tableHeaders.value[alphaResizingState.value.columnIndex]
    const finalWidth = alphaResizingState.value.currentWidth
    alphaResizingState.value.isResizing = false
    document.removeEventListener('mousemove', handleAlphaMouseMove)
    document.removeEventListener('mouseup', stopAlphaResize)

    if (header) {
      await updateColumnWidth(header, finalWidth) // Update store
      debouncedSaveWidths() // Trigger debounced save
    }
  }

  const resetAlphaColumnWidth = async (columnIndex: number) => {
    const header = tableHeaders.value[columnIndex]
    if (header) {
      await resetColumnWidth(header) // Update store
      debouncedSaveWidths() // Trigger save
    }
  } // [cite: 14]

  const startResize = (header: string, event: MouseEvent) => {
    const currentWidth = columnWidths.value[header] || 200
    resizingState.value = {
      isResizing: true,
      header,
      startX: event.clientX,
      startWidth: currentWidth,
      currentWidth: currentWidth,
    }
    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', stopResize)
    event.preventDefault()
  } // [cite: 25]

  const handleMouseMove = (event: MouseEvent) => {
    if (!resizingState.value.isResizing) return
    const delta = event.clientX - resizingState.value.startX
    const newWidth = Math.max(50, resizingState.value.startWidth + delta)
    resizingState.value.currentWidth = newWidth

    const header = resizingState.value.header
    if (header && collectionSchema.value.ui) {
      // Temporarily update local schema for visual feedback
      collectionSchema.value.ui.columnWidths = {
        ...collectionSchema.value.ui.columnWidths,
        [header]: newWidth,
      }
    }
  }

  const stopResize = async () => {
    if (!resizingState.value.isResizing) return
    const header = resizingState.value.header
    const finalWidth = resizingState.value.currentWidth
    resizingState.value.isResizing = false
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', stopResize)

    if (header) {
      await updateColumnWidth(header, finalWidth) // Update store
      debouncedSaveWidths() // Trigger save
    }
  }

  // const resetColumnWidth = async (header: string) => { // Renamed from original resetDataColumnWidth for clarity
  //   await resetColumnWidth(header); // Call store action (make sure names don't clash)
  //   debouncedSaveWidths();
  // }; // [cite: 25]

  // --- Cell Click/Edit Handling ---

  const handleCellClick = (rowIndex: number, header: string, value: any) => {
    // Value param from original template [cite: 52]
    if (isSaving.value || ['_id', 'created_at', 'updated_at'].includes(header)) return

    // Use the original value passed from the template click event
    startEditingCell(rowIndex, header, value) // Use store action

    // Update selectedCell for visual feedback (Excel-like)
    const actualRowNumber = (currentPage.value - 1) * pageSize.value + rowIndex + 1
    const colIndex = tableHeaders.value.indexOf(header)
    selectedCell.value = { colIndex, rowNumber: actualRowNumber } // [cite: 2]

    // Focus logic can be kept if desired
    nextTick(() => {
      // Simplified focus selector - adjust if needed
      const inputElement = scrollContainer.value?.querySelector<
        HTMLInputElement | HTMLTextAreaElement
      >(
        `tr:nth-child(${rowIndex + 1}) td[class*='excel-cell'] textarea, tr:nth-child(${rowIndex + 1}) td[class*='excel-cell'] input`
      )
      inputElement?.focus()
      if (inputElement && (inputElement.tagName === 'TEXTAREA' || inputElement.type === 'text')) {
        inputElement.select()
      }
    })
  } // Combines original template call [cite: 52] with store logic

  // Handle blur event on editable inputs/textareas - uses store action
  const handleEditBlur = async () => {
    setTimeout(async () => {
      const activeElement = document.activeElement
      // Basic check if focus is still within the editing area (might need refinement)
      const isStillEditing =
        editingCell.value &&
        activeElement &&
        (activeElement.closest('.excel-cell-selected') ||
          activeElement.closest('[data-radix-popper-content-wrapper]')) // Consider Radix poppers for Select

      if (editingCell.value && !isStillEditing) {
        await saveEdit() // Use store action
      } else if (!isStillEditing) {
        cancelEdit() // Use store action
      }
    }, 100)
  } // Connects to @blur event [cite: 50]

  // --- Template specific handlers ---
  // Handle view change from ExcelCellReference component
  const handleViewChange = (view: string) => {
    changeView(view) // Use store action
  } // [cite: 4]

  // Handler for pagination component update event
  const onPageChange = (page: number) => {
    setPage(page) // Use store action
  } // [cite: 79]

  // Handlers for delete start/end from actions components
  const handleDeleteStart = (id: string) => {
    dataTableStore.pendingDeleteId = id
  } // [cite: 3, 58, 60]
  const handleDeleteEnd = () => {
    dataTableStore.pendingDeleteId = null
  } // [cite: 4, 59, 60]

  // Expose methods (less common with Pinia but kept if original template needed it)
  defineExpose({
    fetchDocuments: dataTableStore.fetchDocuments,
    fetchCollections: dataTableStore.fetchCollections,
    setCollection: dataTableStore.setCollection,
  })

  const showContextMenu = ref(false)
  const contextMenuPosition = ref({ x: 0, y: 0 })
  const selectedCellInfo = ref<{ rowIndex: number; header: string } | null>(null)

  // methods for context menu
  const handleRightClick = (rowIndex: number, header: string, event: MouseEvent) => {
    event.preventDefault()
    selectedCellInfo.value = { rowIndex, header }

    // Calculate actual row number and column index for highlighting
    const actualRowNumber = (currentPage.value - 1) * pageSize.value + rowIndex + 1
    const colIndex = tableHeaders.value.indexOf(header)
    selectedCell.value = { colIndex, rowNumber: actualRowNumber }

    // Position context menu
    const offset = 55
    contextMenuPosition.value = {
      x: event.clientX,
      y: event.clientY - offset,
    }
    showContextMenu.value = true
  }

  const closeContextMenu = () => {
    showContextMenu.value = false
    selectedCellInfo.value = null
  }

  const pinCell = async () => {
    console.log('pinCell: Function called')
    if (!selectedCellInfo.value) {
      console.warn('pinCell: No cell selected')
      return
    }
    if (isSelectedArchived.value) {
      console.warn('pinCell: Selected document is archived, cannot pin/unpin')
      return
    }
    // Add check for null user
    if (!user.value) {
      console.warn('pinCell: User not authenticated')
      return
    }
    const rowIndex = selectedCellInfo.value.rowIndex
    console.log(`pinCell: Selected row index: ${rowIndex}`)
    const doc = paginatedDocuments.value[rowIndex]
    if (!doc) {
      console.warn(`pinCell: No document found at row index ${rowIndex}`)
      return
    }
    const isPinned = doc.pinned_by?.includes(user.value.id)
    console.log(
      `pinCell: Document ID: ${doc._id.$oid}, Current pin state: ${isPinned ? 'pinned' : 'unpinned'}`
    )
    try {
      if (isPinned) {
        console.log(`pinCell: Document is currently pinned, attempting to unpin`)
        await dataTableStore.unpinDocument(doc._id.$oid)
        console.log(`pinCell: Unpin operation completed`)
      } else {
        console.log(`pinCell: Document is currently unpinned, attempting to pin`)
        await dataTableStore.pinDocument(doc._id.$oid)
        console.log(`pinCell: Pin operation completed`)
      }

      // Add this critical line to refresh documents after pin/unpin operation
      console.log('pinCell: Refreshing documents to ensure pin state consistency across users')
      await dataTableStore.fetchDocuments()
      console.log('pinCell: Document refresh completed')
    } catch (error) {
      console.error('pinCell: Pin/Unpin error:', error)
    }
    console.log('pinCell: Closing context menu')
    closeContextMenu()
  }

  const bookmarkCell = () => {
    if (!selectedCellInfo.value) return
    console.log('Bookmarking cell:', selectedCellInfo.value)
    closeContextMenu()
  }

  // Add window click listener (in onMounted)
  onMounted(() => {
    window.addEventListener('click', closeContextMenu)
  })

  // Add cleanup (if using beforeUnmount)
  onBeforeUnmount(() => {
    window.removeEventListener('click', closeContextMenu)
  })
  const isSelectedArchived = computed(() => {
    console.log('Computing isSelectedArchived')

    if (!selectedCellInfo.value) {
      console.log('No selectedCellInfo, returning false')
      return false
    }

    const rowIndex = selectedCellInfo.value.rowIndex
    console.log('Selected row index:', rowIndex)

    const document = paginatedDocuments.value[rowIndex]
    console.log('Selected document:', document)

    const isArchived = document?.is_archive === true
    console.log('Is document archived:', isArchived)

    return isArchived
  })

  const selectedDocumentIsPinned = computed(() => {
    if (!selectedCellInfo.value) return false
    if (!user.value) return false // Add this check for null user

    const doc = paginatedDocuments.value[selectedCellInfo.value.rowIndex]
    return doc?.pinned_by?.includes(user.value.id)
  })

  const togglePinStatus = async (docId: string, currentPinStatus: boolean): Promise<void> => {
    try {
      if (currentPinStatus) {
        await dataTableStore.unpinDocument(docId)
      } else {
        await dataTableStore.pinDocument(docId)
      }
    } catch (error) {
      console.error('Error toggling pin status:', error)
    }
  }

  // --- New Computed Property ---
  const pinnedDocuments = computed(() => {
    if (!user.value) return []
    return documents.value.filter((doc) => doc.pinned_by?.includes(user.value?.id))
  })

  // --- New Watcher ---
  watch(
    pinnedDocuments,
    (newPinnedDocs) => {
      console.log('Pinned Documents:', JSON.parse(JSON.stringify(newPinnedDocs)))
    },
    { immediate: true, deep: true }
  )

  const highlightedDocumentId = ref<string | null>(null)
  let highlightTimeout: ReturnType<typeof setTimeout> | null = null

  const handleDocumentNavigation = (docId: string) => {
    // Clear existing timeout if any
    if (highlightTimeout) clearTimeout(highlightTimeout)

    // Find document index in full dataset
    const index = documents.value.findIndex((doc) => doc._id.$oid === docId)
    if (index === -1) return

    // Calculate and set correct page
    const page = Math.ceil((index + 1) / pageSize.value)
    setPage(page)

    // Set highlight and auto-clear after 3s
    highlightedDocumentId.value = docId
    highlightTimeout = setTimeout(() => {
      highlightedDocumentId.value = null
    }, 2000)

    // Scroll to row after DOM update
    nextTick(() => {
      const row = scrollContainer.value?.querySelector(`[data-document-id="${docId}"]`)
      if (row) {
        row.scrollIntoView({
          behavior: 'smooth',
          block: 'nearest',
          inline: 'nearest',
        })

        // Add visual pulse effect
        row.classList.add('highlight-pulse')
        setTimeout(() => row.classList.remove('highlight-pulse'), 1000)
      }
    })
  }
</script>

<template>
  <!-- MongoDBDataTable main div -->
  <div
    :class="isSidebarOpen ? 'ml-[280px]' : 'ml-0'"
    class="transition-all duration-300 ease-in-out"
  >
    <MongoDBDataTableNavbar
      :isSplitActive="isSplit"
      class="sticky top-0 z-50"
    />
    <div class="excel-container w-full">
      <StickyLeftSidebar
        :isOpen="isSidebarOpen"
        :pinnedDocuments="pinnedDocuments"
        @toggle="isSidebarOpen = !isSidebarOpen"
        @navigate="handleDocumentNavigation"
      />
      <div
        v-if="errorMessage"
        class="fixed top-4 left-4 right-4 z-[9999] mx-4 my-4 p-4 bg-red-100 text-red-700 rounded-lg shadow-xl border-2 border-red-300 break-words"
      >
        {{ errorMessage }}
        <Button
          @click="closeErrorManual"
          variant="ghost"
          size="sm"
          class="absolute right-3 top-3 p-1 h-6 w-6 text-red-700 hover:bg-red-200"
        >
          <Cross2Icon class="h-3 w-3" />
        </Button>
      </div>

      <div
        v-if="isLoading"
        class="flex justify-center my-8"
      >
        <ReloadIcon class="h-8 w-8 animate-spin text-gray-500" />
      </div>

      <div
        ref="scrollContainer"
        class="w-full overflow-auto table-scroll-container"
      >
        <ExcelCellReference
          :is-sidebar-open="isSidebarOpen"
          :selected-cell="selectedCell"
          :selected-rows="selectedRows"
          :collection-name="collectionName"
          :documents="documents"
          :current-page="currentPage"
          :page-size="pageSize"
          :current-view="currentView"
          @document-deleted="fetchDocuments"
          @reset-selection="resetSelection"
          @delete-start="handleDeleteStart"
          @delete-end="handleDeleteEnd"
          @view-change="handleViewChange"
        />
        <table
          class="mt-10 excel-table"
          :style="{ width: `${totalTableWidth}px` }"
        >
          <TableHeader>
            <TableRow class="excel-header-row">
              <TableHead
                class="excel-column-checkbox-selector"
                :style="{
                  width: '40px',
                  minWidth: '40px',
                  maxWidth: '40px',
                }"
              >
                <input
                  type="checkbox"
                  :checked="allSelected"
                  @change="allSelected = !allSelected"
                  :disabled="documents.length === 0"
                  class="excel-checkbox"
                />
              </TableHead>

              <TableHead
                class="excel-column-checkbox"
                :style="{
                  width: '30px',
                  minWidth: '30px',
                  maxWidth: '30px',
                }"
              >
                @
              </TableHead>
              <TableHead
                v-for="(letter, index) in columnLetters"
                :key="`letter-${index}`"
                class="excel-column-letter relative"
                :style="{
                  width:
                    alphaResizingState.isResizing && alphaResizingState.columnIndex === index
                      ? `${alphaResizingState.currentWidth}px`
                      : `${columnWidths[tableHeaders[index]] || 200}px`,
                }"
              >
                <div class="flex items-center justify-center">
                  <span class="excel-letter">{{ letter }}</span>
                  <div
                    class="excel-resizer absolute right-0 top-0"
                    :class="[
                      alphaResizingState.isResizing && alphaResizingState.columnIndex === index
                        ? 'excel-resizer-active'
                        : '',
                    ]"
                    @mousedown="startAlphaResize(index, $event)"
                    @dblclick="resetAlphaColumnWidth(index)"
                  ></div>
                </div>
              </TableHead>
              <TableHead class="excel-column-letter excel-actions-header w-24"> </TableHead>
            </TableRow>
          </TableHeader>
          <TableHeader>
            <TableRow>
              <TableHead
                class="excel-column-checkbox-selector"
                :style="{
                  width: '40px',
                  minWidth: '40px',
                  maxWidth: '40px',
                }"
              >
                ***
              </TableHead>

              <TableHead
                class="excel-row-number-header"
                :style="{
                  width: numberColumnWidth,
                  minWidth: numberColumnWidth,
                  maxWidth: numberColumnWidth,
                }"
              >
                &-
              </TableHead>
              <TableHead
                v-for="header in tableHeaders"
                :key="header"
                class="excel-column-header font-bold text-black relative"
                :class="{ 'error-column-header': header === errorColumn && isAdding }"
                :style="{
                  width:
                    resizingState.isResizing && resizingState.header === header
                      ? `${resizingState.currentWidth}px`
                      : `${columnWidths[header] || 200}px`,
                }"
              >
                <div class="flex items-center justify-between">
                  <span>
                    {{ header }}
                    <span
                      v-if="isFieldRequired(header)"
                      class="text-red-500"
                      >*</span
                    >
                  </span>
                  <div
                    class="excel-resizer absolute right-0 top-0"
                    :class="[
                      resizingState.isResizing && resizingState.header === header
                        ? 'excel-resizer-active'
                        : '',
                    ]"
                    @mousedown="startResize(header, $event)"
                    @dblclick="resetColumnWidth(header)"
                  ></div>
                </div>
              </TableHead>
              <TableHead
                class="excel-column-header excel-actions-header select-none"
                :style="{ width: '30px' }"
              >
                Actions
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <template v-if="documents.length > 0">
              <TableRow
                v-for="(doc, rowIndex) in paginatedDocuments"
                :key="doc._id.$oid"
                :data-document-id="doc._id.$oid"
                class="excel-data-row"
                :class="{
                  'highlight-row': highlightedDocumentId === doc._id.$oid,
                  'bg-red-100 border-2 border-red-500 text-red-800':
                    doc._id.$oid === pendingDeleteId,
                  'selected-row bg-blue-100':
                    selectedRows.has(doc._id.$oid) && doc._id.$oid !== pendingDeleteId,
                }"
              >
                <TableCell
                  class="excel-column-checkbox-selector"
                  :style="{
                    width: '40px',
                    minWidth: '40px',
                    maxWidth: '40px',
                  }"
                >
                  <input
                    type="checkbox"
                    :checked="selectedRows.has(doc._id.$oid)"
                    @change="toggleRow(doc._id.$oid)"
                    class="excel-checkbox"
                  />
                </TableCell>
                <TableCell
                  class="excel-row-number"
                  :style="{
                    width: numberColumnWidth,
                    minWidth: numberColumnWidth,
                    maxWidth: numberColumnWidth,
                  }"
                >
                  <div
                    class="relative inline-block w-full h-full cursor-pointer"
                    @click.stop="
                      togglePinStatus(doc._id.$oid, user && doc.pinned_by?.includes(user.id))
                    "
                    :class="{ 'hover:bg-gray-100': !doc.is_archive }"
                    :title="
                      doc.is_archive
                        ? 'Cannot pin/unpin archived items'
                        : user && doc.pinned_by?.includes(user.id)
                          ? 'Click to unpin'
                          : 'Click to pin'
                    "
                  >
                    <span
                      v-if="user && doc.pinned_by?.includes(user.id)"
                      class="text-xl absolute top-1 left-[5px] -translate-y-1/2"
                    >
                      📌
                    </span>
                    <span class="">{{ (currentPage - 1) * pageSize + rowIndex + 1 }}</span>
                  </div>
                </TableCell>
                <TableCell
                  v-for="header in tableHeaders"
                  :key="`${doc._id.$oid}-${header}`"
                  class="excel-cell"
                  :class="{
                    'error-column-cell': header === errorColumn,
                    'excel-cell-selected':
                      editingCell?.rowIndex === rowIndex && editingCell?.header === header,
                    'excel-cell-context-selected':
                      selectedCellInfo?.rowIndex === rowIndex &&
                      selectedCellInfo?.header === header,
                  }"
                  @contextmenu.prevent="handleRightClick(rowIndex, header, $event)"
                >
                  <div class="h-full">
                    <div
                      v-if="editingCell?.rowIndex === rowIndex && editingCell?.header === header"
                      class="h-full"
                    >
                      <div
                        v-if="getSchemaInfo(header).bsonType === 'bool'"
                        class="flex items-center justify-center h-full p-2"
                      >
                        <input
                          type="checkbox"
                          v-model="editValue"
                          @change="saveEdit"
                          class="excel-checkbox"
                        />
                      </div>
                      <div
                        v-else-if="isReferenceField(header)"
                        class="p-1"
                      >
                        <Select
                          v-model="editValue"
                          @update:modelValue="saveEdit"
                          class="excel-select"
                        >
                          <SelectTrigger>
                            <SelectValue
                              :placeholder="`Select ${getReferencedCollection(header)}`"
                              :model-value="editValue"
                            />
                          </SelectTrigger>
                          <SelectContent>
                            <ScrollArea class="h-48">
                              <div class="p-1">
                                <Input
                                  v-model="searchQuery[header]"
                                  placeholder="Search..."
                                  class="mb-2 excel-input"
                                />
                                <div
                                  v-if="loadingReferences[getReferencedCollection(header)!]"
                                  class="text-center p-2"
                                >
                                  <ReloadIcon class="h-4 w-4 animate-spin" />
                                </div>
                                <div v-else-if="filteredOptions(header).length">
                                  <SelectItem
                                    v-for="option in filteredOptions(header)"
                                    :key="option.id"
                                    :value="option.id"
                                  >
                                    {{ option.label }}
                                  </SelectItem>
                                </div>
                                <div
                                  v-else
                                  class="text-sm text-gray-500 px-2 py-1"
                                >
                                  No options found
                                </div>
                              </div>
                            </ScrollArea>
                          </SelectContent>
                        </Select>
                      </div>
                      <Input
                        v-else-if="getSchemaInfo(header).bsonType === 'date'"
                        type="datetime-local"
                        v-model="editValue"
                        @blur="handleEditBlur"
                        class="excel-input excel-date-input"
                      />
                      <textarea
                        v-else
                        v-model="editValue"
                        @blur="handleEditBlur"
                        @keyup.ctrl.enter="saveEdit"
                        @keyup.esc="cancelEdit"
                        class="excel-textarea"
                        rows="1"
                      ></textarea>
                    </div>
                    <div
                      v-else
                      class="excel-cell-content"
                      :class="{
                        'excel-cell-editable': !['_id', 'created_at', 'updated_at'].includes(
                          header
                        ),
                        'excel-cell-readonly': ['_id', 'created_at', 'updated_at'].includes(header),
                      }"
                      @click="handleCellClick(rowIndex, header, doc[header])"
                    >
                      <div
                        v-if="getSchemaInfo(header).bsonType === 'bool'"
                        class="flex justify-center"
                      >
                        <input
                          type="checkbox"
                          :checked="doc[header]"
                          disabled
                          class="excel-checkbox"
                        />
                      </div>
                      <div
                        v-else-if="isReferenceField(header)"
                        class="excel-reference-value"
                      >
                        <span v-if="loadingReferences[getReferencedCollection(header)!]">...</span>
                        <span v-else>{{
                          getReferenceLabel(header, doc[header]) || doc[header]
                        }}</span>
                      </div>
                      <template v-else-if="['created_at', 'updated_at'].includes(header)">
                        <span class="excel-timestamp">
                          {{ formatSchemaValue(doc[header], getSchemaInfo(header).bsonType) }}
                        </span>
                      </template>
                      <template v-else-if="header === '_id'">
                        <span>{{ doc[header]?.$oid || doc[header] }}</span>
                      </template>
                      <template v-else>
                        {{ formatSchemaValue(doc[header], getSchemaInfo(header).bsonType) }}
                      </template>
                    </div>
                  </div>
                </TableCell>
                <TableActions
                  :collection-name="collectionName"
                  :document-id="doc._id.$oid"
                  :row-number="(currentPage - 1) * pageSize + rowIndex + 1"
                  @deleted="fetchDocuments"
                  @delete-start="handleDeleteStart"
                  @delete-end="handleDeleteEnd"
                />
                <StickyTableActions
                  :collection-name="collectionName"
                  :document-id="doc._id.$oid"
                  :row-number="(currentPage - 1) * pageSize + rowIndex + 1"
                  :target-ref="scrollContainer"
                  :is-last-row="rowIndex === paginatedDocuments.length - 1"
                  :is-single-row="paginatedDocuments.length === 1"
                  @deleted="fetchDocuments"
                  @delete-start="handleDeleteStart"
                  @delete-end="handleDeleteEnd"
                />
              </TableRow>

              <!-- Context Menu -->
              <div
                v-if="showContextMenu"
                class="fixed z-50 bg-white shadow-lg border rounded-md p-1 min-w-[120px] context-menu"
                :style="{
                  left: `${contextMenuPosition.x}px`,
                  top: `${contextMenuPosition.y}px`,
                }"
                @click="closeContextMenu"
              >
                <div
                  class="flex items-center px-3 py-1.5 text-sm rounded-sm relative tooltip-container"
                  :class="[
                    isSelectedArchived
                      ? 'text-gray-400 cursor-not-allowed'
                      : 'hover:bg-gray-100 cursor-pointer text-gray-700',
                  ]"
                  @click="pinCell"
                  @mouseenter="isSelectedArchived && (showPinTooltip = true)"
                  @mouseleave="showPinTooltip = false"
                >
                  <span>
                    <template v-if="selectedDocumentIsPinned">📌 Unpin this item</template>
                    <template v-else>📌 Pin this item</template>
                  </span>
                  <!-- Custom tooltip -->
                  <div
                    v-if="isSelectedArchived && showPinTooltip"
                    class="custom-tooltip absolute bg-gray-800 text-white text-xs rounded py-1 px-2 left-0 bottom-full mb-1 whitespace-nowrap pointer-events-none z-50"
                  >
                    You cannot pin an archived item
                    <div
                      class="tooltip-arrow absolute top-full left-4 w-2 h-2 bg-gray-800 transform rotate-45"
                    ></div>
                  </div>
                </div>
                <div
                  class="flex items-center px-3 py-1.5 text-sm hover:bg-gray-100 rounded-sm cursor-pointer"
                  @click="bookmarkCell"
                >
                  🔖 Bookmark
                </div>
              </div>

              <!-- End of Context Menu -->
            </template>

            <TableRow
              v-if="isAdding"
              class="excel-new-row"
              :class="['excel-new-row', { 'excel-new-row-error': addingRowError }]"
            >
              <TableCell
                class="excel-column-checkbox-selector"
                :style="{
                  width: '40px',
                  minWidth: '40px',
                  maxWidth: '40px',
                }"
              >
                <input
                  type="checkbox"
                  disabled
                  class="excel-checkbox"
                />
              </TableCell>
              <TableCell class="excel-row-number"> {{ documents.length + 1 }} </TableCell>
              <TableCell
                v-for="header in tableHeaders"
                :key="`new-${header}`"
                class="excel-cell"
                :class="{ 'error-column-cell': header === errorColumn }"
              >
                <span
                  v-if="['created_at', 'updated_at'].includes(header)"
                  class="excel-timestamp"
                >
                  (auto-generated)
                </span>

                <div
                  v-else-if="header !== '_id' && getSchemaInfo(header).bsonType === 'bool'"
                  class="flex items-center justify-center"
                >
                  <input
                    type="checkbox"
                    v-model="newDocument[header]"
                    class="excel-checkbox"
                  />
                </div>
                <div
                  v-else-if="header !== '_id' && isReferenceField(header)"
                  class="h-8"
                >
                  <Select
                    v-model="newDocument[header]"
                    class="excel-select"
                  >
                    <SelectTrigger class="h-8">
                      <SelectValue :placeholder="`Select`" />
                    </SelectTrigger>
                    <SelectContent>
                      <div
                        v-if="loadingReferences[getReferencedCollection(header)!]"
                        class="p-2"
                      >
                        <ReloadIcon class="h-4 w-4 animate-spin mx-auto" />
                      </div>
                      <ScrollArea
                        v-else
                        class="h-48"
                      >
                        <Input
                          v-model="searchQuery[header]"
                          placeholder="Search..."
                          class="mb-1 mx-1 excel-input"
                        />
                        <SelectItem
                          v-for="option in filteredOptions(header)"
                          :key="option.id"
                          :value="option.id"
                        >
                          {{ option.label }}
                        </SelectItem>
                      </ScrollArea>
                    </SelectContent>
                  </Select>
                </div>
                <Input
                  v-else-if="header !== '_id'"
                  v-model="newDocument[header]"
                  :type="getSchemaInfo(header).bsonType === 'date' ? 'datetime-local' : 'text'"
                  class="excel-input"
                  :class="{ 'ring-2 ring-red-500': header === errorColumn }"
                />
                <span
                  v-else
                  class="excel-auto-id"
                  >(auto)</span
                >
              </TableCell>
              <TableCell class="excel-cell text-center">
                <Button
                  variant="ghost"
                  @click="saveNewDocument"
                  size="sm"
                  class="px-0 -ml-1"
                  :disabled="isSaving"
                >
                  <ReloadIcon
                    v-if="isSaving"
                    class="h-4 w-4 animate-spin"
                  />
                  <span v-else>💾</span>
                </Button>
              </TableCell>
            </TableRow>

            <TableRow
              v-if="!isAdding"
              class="excel-add-row"
              @click="startAdding"
            >
              <TableCell
                :colspan="tableHeaders.length + 3"
                class="excel-add-cell"
              >
                <div class="inline-flex items-center gap-2 excel-add-button">
                  <PlusCircledIcon class="h-4 w-4" />
                  <span class="text-sm">
                    {{ documents.length === 0 ? 'Add first document' : 'Add new document' }}
                  </span>
                </div>
              </TableCell>
            </TableRow>
          </TableBody>
        </table>

        <div
          v-if="isAdding"
          class="sticky top-2 left-2 z-20 p-3 shadow-lg flex items-center space-x-2 w-auto rounded-md bg-green-50"
        >
          <Button
            @click="saveNewDocument"
            size="sm"
            class="bg-green-600 hover:bg-green-700 text-white"
            :disabled="isSaving"
          >
            <ReloadIcon
              v-if="isSaving"
              class="w-4 h-4 animate-spin"
            />
            <PlusCircledIcon
              v-else
              class="w-4 h-4 mr-1"
            />
            Save
          </Button>
          <Button
            @click="cancelAdding"
            variant="outline"
            size="sm"
            class="border-green-600 text-green-700 hover:bg-green-100"
            :disabled="isSaving"
          >
            <Cross2Icon class="w-4 h-4 mr-1" /> Cancel
          </Button>
        </div>

        <div
          v-if="totalPages > 1"
          class="excel-pagination"
        >
          <Pagination
            :page="currentPage"
            :itemsPerPage="pageSize"
            :total="documents.length"
            @update:page="onPageChange"
            :siblingCount="1"
          >
            <PaginationList>
              <PaginationListItem :value="1">
                <PaginationFirst
                  :disabled="currentPage === 1"
                  @click="setPage(1)"
                  class="excel-pagination-button"
                />
              </PaginationListItem>
              <PaginationListItem :value="Math.max(1, currentPage - 1)">
                <PaginationPrev
                  :disabled="currentPage === 1"
                  @click="setPage(currentPage - 1)"
                  class="excel-pagination-button"
                />
              </PaginationListItem>
              <PaginationListItem :value="Math.min(totalPages, currentPage + 1)">
                <PaginationNext
                  :disabled="currentPage === totalPages"
                  @click="setPage(currentPage + 1)"
                  class="excel-pagination-button"
                />
              </PaginationListItem>
              <PaginationListItem :value="totalPages">
                <PaginationLast
                  :disabled="currentPage === totalPages"
                  @click="setPage(totalPages)"
                  class="excel-pagination-button"
                />
              </PaginationListItem>
            </PaginationList>
          </Pagination>
        </div>

        <div
          v-if="!isAdding"
          class="excel-footer"
        >
          <span class="excel-page-size-label">Rows per page:</span>
          <Select
            :modelValue="String(pageSize)"
            @update:modelValue="(val) => setPageSize(Number(val))"
            class="excel-page-size-select"
          >
            <SelectTrigger class="w-16"> <SelectValue /> </SelectTrigger>
            <SelectContent>
              <SelectItem value="5">5</SelectItem> <SelectItem value="10">10</SelectItem>
              <SelectItem value="20">20</SelectItem> <SelectItem value="50">50</SelectItem>
              <SelectItem value="100">100</SelectItem>
            </SelectContent>
          </Select>

          <span class="excel-status-info">
            Showing {{ paginatedDocuments.length ? (currentPage - 1) * pageSize + 1 : 0 }} to
            {{ (currentPage - 1) * pageSize + paginatedDocuments.length }} of
            {{ documents.length }} entries
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
  .highlight-row {
    position: relative;
    animation: highlight-fade 2s forwards;
    box-shadow: 0 0 0 2px #fde047; /* Outline yellow */
    outline: 2px solid #fde047; /* Same as box shadow */
    background-color: #fff9c4; /* Lighter yellow background */
  }

  @keyframes highlight-fade {
    0% {
      box-shadow: 0 0 0 3px #fde047;
    }
    70% {
      box-shadow: 0 0 0 3px #fde047;
    }
    100% {
      box-shadow: 0 0 0 3px transparent;
    }
  }

  .highlight-pulse {
    animation: pulse-highlight 1s ease-in-out;
  }

  @keyframes pulse-highlight {
    0% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.02);
    }
    100% {
      transform: scale(1);
    }
  }

  /* Context Menu */
  .tooltip-container {
    position: relative;
  }

  .custom-tooltip {
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
    animation: fadeIn 0.2s ease-in-out;
  }

  .tooltip-arrow {
    position: absolute;
    top: 100%;
    left: 10px;
    margin-top: -4px;
    width: 8px;
    height: 8px;
    transform: rotate(45deg);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .excel-cell-context-selected {
    outline: 2px solid #217346;
    outline-offset: -2px;
    position: relative;
    z-index: 1;
    overflow: visible;
  }

  .context-menu {
    transform: translateY(-100%); /* Move menu up by its own height */
    pointer-events: auto; /* Ensure menu remains interactive */
  }

  /* small arrow */
  .context-menu::before {
    content: '';
    position: absolute;
    top: -5px; /* Change from bottom to top */
    left: 10px;
    width: 0;
    height: 0;
    border-left: 5px solid transparent;
    border-right: 5px solid transparent;
    border-bottom: 5px solid white; /* Change from border-top to border-bottom */
    filter: drop-shadow(0 -1px 1px rgba(0, 0, 0, 0.1)); /* Adjust shadow direction */
  }

  /* --- Style Section (From old_codes.txt, using <style scoped>) --- */
  .selected-row {
    outline: 2px solid #2196f3; /* [cite: 84] */
    border: 2px solid #2196f3; /* [cite: 85] */
    outline-offset: -1px;
    position: relative;
  }

  .bg-red-100 {
    /* Style for pending delete row */
    outline: none !important; /* [cite: 85] */
  }

  /* Excel-inspired container */
  .excel-container {
    font-family: 'Segoe UI', Arial, sans-serif; /* [cite: 86] */
    border: 1px solid #d4d4d8; /* [cite: 86] */
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05); /* [cite: 87] */
    border-radius: 2px;
    background-color: #ffffff; /* [cite: 87] */
  }

  /* Excel table styling */
  .excel-table {
    table-layout: fixed; /* [cite: 88] */
    min-width: fit-content; /* [cite: 88] */
    border-collapse: collapse; /* Add this */
  }

  /* Excel header row */
  .excel-header-row {
    background-color: #f3f3f3; /* [cite: 88] */
  }

  /* Excel column headers */
  .excel-column-header {
    background-color: #f3f3f3; /* [cite: 89] */
    border: 1px solid #d0d0d0; /* [cite: 89] */
    padding: 6px 8px; /* [cite: 89] */
    font-weight: 600; /* [cite: 89] */
    font-size: 14px; /* [cite: 89] */
    color: #000000; /* [cite: 90] */
    position: relative; /* [cite: 90] */
    text-align: left; /* [cite: 90] */
    height: 32px; /* Added height */
    box-sizing: border-box; /* Added box-sizing */
    vertical-align: middle; /* Center vertically */
  }

  /* Excel column letter headers (A, B, C) */
  .excel-column-letter {
    background-color: #e6e6e6; /* [cite: 90] */
    border: 1px solid #d0d0d0; /* [cite: 91] */
    padding: 4px 8px; /* [cite: 91] */
    font-weight: 600; /* [cite: 91] */
    font-size: 14px; /* [cite: 91] */
    color: #616161; /* [cite: 91] */
    text-align: center; /* [cite: 91] */
    height: 28px; /* Added height */
    box-sizing: border-box; /* Added box-sizing */
    vertical-align: middle; /* Center vertically */
  }

  /* Update sticky positioning for new column */
  .excel-column-checkbox-selector {
    position: sticky; /* [cite: 92] */
    left: 0; /* [cite: 92] */
    z-index: 5; /* [cite: 92] */
    background-color: #f3f3f3; /* [cite: 92] */ /* Match header */
    border: 1px solid #d0d0d0; /* Added border */
    text-align: center; /* Center checkbox */
    vertical-align: middle; /* Center checkbox */
    box-sizing: border-box;
  }
  /* Need background for sticky cells in body */
  .excel-data-row .excel-column-checkbox-selector {
    background-color: #ffffff; /* White for data rows */
  }
  .excel-data-row:hover .excel-column-checkbox-selector {
    background-color: #edf5fd; /* Match row hover */
  }
  .excel-data-row.selected-row .excel-column-checkbox-selector {
    background-color: #ebf8ff; /* Match selected row (adjust color) */
  }
  .excel-data-row.bg-red-100 .excel-column-checkbox-selector {
    background-color: #fee2e2; /* Match delete row */
  }

  /* Adjust row number positioning */
  .excel-row-number-header {
    /* Applied to TH */
    position: sticky; /* [cite: 98, 100] */
    left: 40px; /* [cite: 98, 101] */ /* Match selector width */
    z-index: 4; /* [cite: 99, 101] */
    background-color: #f3f3f3; /* [cite: 99, 101] */
    border: 1px solid #d0d0d0; /* [cite: 100, 102] */
    /* outline: 1px solid #d0d0d0; */ /* [cite: 100, 103] Outline removed, use border */
    text-align: center; /* Center text */
    vertical-align: middle; /* Center text */
    box-sizing: border-box;
    width: 30px !important; /* [cite: 124] */
    min-width: 30px !important; /* [cite: 124] */
    max-width: 30px !important; /* [cite: 124] */
    padding: 6px 0px; /* Adjust padding */
    font-weight: 600; /* [cite: 97] */
    font-size: 14px; /* [cite: 97] */
    color: #616161; /* [cite: 97] */
  }
  .excel-row-number {
    /* Applied to TD */
    position: sticky; /* [cite: 104] */
    left: 40px; /* [cite: 104] */ /* Match selector width */
    z-index: 2; /* [cite: 105] */
    background-color: #f3f3f3; /* [cite: 105] */
    border: 1px solid #d0d0d0; /* [cite: 105] */
    /* outline: 1px solid #d0d0d0; */ /* [cite: 106] Outline removed, use border */
    text-align: center;
    vertical-align: middle;
    box-sizing: border-box;
    width: 30px !important; /* [cite: 124] */
    min-width: 30px !important; /* [cite: 124] */
    max-width: 30px !important; /* [cite: 124] */
    font-size: 14px; /* Match headers */
    color: #616161; /* Match headers */
  }
  /* Hover state for row numbers */
  .excel-data-row:hover .excel-row-number {
    background-color: #edf5fd; /* [cite: 140] */ /* Match row hover color */
  }
  .excel-data-row.selected-row .excel-row-number {
    background-color: #ebf8ff; /* Match selected row (adjust color) */
  }
  .excel-data-row.bg-red-100 .excel-row-number {
    background-color: #fee2e2; /* Match delete row */
  }

  /* Original selector column TH style (no longer needed?) */
  /* .excel-column-checkbox {
  position: sticky; [cite: 95]
  left: 40px; [cite: 95]
  z-index: 4; [cite: 95]
  background-color: #e6e6e6; [cite: 96]
  border: 1px solid #d0d0d0; [cite: 96]
  padding: 4px 8px; [cite: 96]
  font-weight: 600; [cite: 97]
  font-size: 14px; [cite: 97]
  color: #616161; [cite: 97]
  text-align: center; [cite: 97]
  outline: 1px solid #d0d0d0; [cite: 97]
} */

  .excel-letter {
    font-weight: 700; /* [cite: 97] */
  }

  /* Excel actions header */
  .excel-actions-header {
    /* Applied to TH */
    background-color: #f3f3f3; /* [cite: 106] */
    border: 1px solid #d0d0d0; /* [cite: 106] */
    text-align: center; /* [cite: 106] */
    position: sticky; /* Added sticky */
    right: 0; /* Added sticky */
    z-index: 4; /* Added sticky */
    width: 60px !important; /* [cite: 125] */
    min-width: 60px !important; /* [cite: 125] */
    max-width: 60px !important; /* [cite: 125] */
    box-sizing: border-box;
    vertical-align: middle;
  }
  .excel-actions-cell {
    /* Applied to TD */
    border: 1px solid #d0d0d0;
    position: sticky; /* Added sticky */
    right: 0; /* Added sticky */
    z-index: 2; /* Added sticky */
    width: 60px !important; /* [cite: 125] */
    min-width: 60px !important; /* [cite: 125] */
    max-width: 60px !important; /* [cite: 125] */
    background-color: #ffffff; /* Need background for sticky */
    text-align: center;
    vertical-align: middle;
    box-sizing: border-box;
  }
  /* Hover/Selected states for actions cell */
  .excel-data-row:hover .excel-actions-cell {
    background-color: #edf5fd; /* Match row hover */
  }
  .excel-data-row.selected-row .excel-actions-cell {
    background-color: #ebf8ff; /* Match selected row (adjust color) */
  }
  .excel-data-row.bg-red-100 .excel-actions-cell {
    background-color: #fee2e2; /* Match delete row */
  }

  /* Excel data rows */
  .excel-data-row:hover {
    background-color: #edf5fd; /* [cite: 107] */
  }

  /* Excel data cell */
  .excel-cell {
    border: 1px solid #d0d0d0; /* [cite: 107] */
    padding: 0; /* [cite: 108] */ /* Reset padding for inputs */
    font-size: 14px; /* [cite: 108] */
    color: #212121; /* [cite: 108] */
    position: relative; /* [cite: 108] */
    height: 40px; /* [cite: 108, 116] */ /* Set consistent height */
    box-sizing: border-box; /* Add box-sizing */
    vertical-align: middle; /* Center content */
    overflow: hidden; /* Hide overflow */
    white-space: nowrap; /* Prevent wrapping */
    text-overflow: ellipsis; /* Add ellipsis */
  }
  .excel-cell div {
    /* Ensure divs within cells allow vertical centering */
    display: flex;
    align-items: center;
    height: 100%;
  }

  /* Excel cell content */
  .excel-cell-content {
    padding: 6px 8px; /* [cite: 109] */ /* Restore padding for read-only */
    min-height: 40px; /* [cite: 109] */
    width: 100%; /* Ensure it fills cell */
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    display: block; /* Override flex from div rule */
  }

  /* Excel cell editable */
  .excel-cell-editable {
    cursor: pointer; /* [cite: 110] */
  }

  .excel-cell-editable:hover {
    background-color: #e8f3fd; /* [cite: 110] */
  }

  /* Excel cell readonly */
  .excel-cell-readonly {
    cursor: not-allowed; /* [cite: 111] */
    opacity: 0.8; /* [cite: 111] */
    background-color: #f9f9f9; /* [cite: 111] */
  }

  /* Excel cell selected - active cell styling */
  .excel-cell-selected {
    outline: 2px solid #217346; /* [cite: 112] */
    outline-offset: -2px; /* [cite: 112] */
    position: relative; /* [cite: 112] */
    z-index: 1; /* [cite: 113] */
    overflow: visible; /* Allow input to overflow slightly if needed */
  }

  /* Excel textarea */
  .excel-textarea {
    width: 100%; /* [cite: 113] */
    height: 100%; /* [cite: 113] */
    padding: 6px 8px; /* [cite: 113] */
    font-family: 'Segoe UI', Arial, sans-serif; /* [cite: 113] */
    font-size: 14px; /* [cite: 114] */
    border: none; /* [cite: 114] */
    resize: none; /* [cite: 114] */
    min-height: 40px; /* [cite: 114] */
    outline: none; /* [cite: 115] */
    box-shadow: none; /* [cite: 115] */
    overflow: hidden; /* [cite: 115] */
    box-sizing: border-box; /* Added */
    background-color: white; /* Ensure background */
    vertical-align: top; /* Align text top */
  }

  /* Excel input */
  .excel-input {
    height: 100%; /* [cite: 118] */
    width: 100%; /* Added */
    min-height: 32px; /* [cite: 118] */ /* Ensure min height */
    border-radius: 0; /* [cite: 118] */
    border: none; /* [cite: 119] */
    box-shadow: none; /* [cite: 119] */
    font-size: 14px; /* [cite: 119] */
    padding: 4px 6px; /* [cite: 119] */
    outline: none; /* Added */
    box-sizing: border-box; /* Added */
    background-color: white; /* Ensure background */
  }

  .excel-input:focus-visible {
    outline: none; /* [cite: 119] */
    box-shadow: none; /* [cite: 120] */
    border: none; /* [cite: 120] */
    /* ring: none; */ /* [cite: 120] */ /* Removed ring */
  }

  /* Excel date input */
  .excel-date-input {
    padding: 2px 4px; /* [cite: 120] */
    font-size: 14px; /* [cite: 120] */
  }

  /* Excel checkbox */
  .excel-checkbox {
    height: 16px; /* [cite: 120] */
    width: 16px; /* [cite: 121] */
    cursor: pointer; /* [cite: 121] */
    accent-color: #217346; /* [cite: 121] */
    vertical-align: middle; /* Align checkbox */
  }

  /* Excel reference value */
  .excel-reference-value {
    color: #0066cc; /* [cite: 121] */
    cursor: pointer; /* [cite: 121] */
    text-decoration: underline; /* Added underline */
  }

  /* Excel timestamp value */
  .excel-timestamp {
    color: #666666; /* [cite: 122] */
    font-style: italic; /* [cite: 122] */
    font-size: 12px; /* [cite: 122] */ /* Slightly smaller */
    display: block; /* Ensure it takes space */
    line-height: normal; /* Reset line height */
  }

  /* Excel auto ID */
  .excel-auto-id {
    color: #888888; /* [cite: 123] */
    font-style: italic; /* [cite: 123] */
    padding: 0 8px; /* [cite: 123] */
    font-size: 12px; /* [cite: 123] */ /* Slightly smaller */
    display: block;
    line-height: normal;
  }

  /* Excel new row */
  .excel-new-row {
    background-color: #e8f5e9; /* [cite: 126] */
  }
  .excel-new-row .excel-cell {
    height: 40px; /* [cite: 116] */
    overflow: visible; /* [cite: 116] */ /* Allow select dropdown */
    vertical-align: middle; /* Center vertically */
  }
  .excel-new-row .excel-input {
    overflow: hidden; /* [cite: 117] */
    height: 100%; /* Ensure input fills cell */
    background-color: white;
    border: 1px solid #d0d0d0; /* Add border for clarity */
  }
  .excel-new-row .excel-select > button {
    /* Target trigger */
    height: 100%;
    background-color: white;
    border: 1px solid #d0d0d0;
  }

  /* Excel cancel button (if used) */
  .excel-cancel-button {
    color: #666666; /* [cite: 126] */
    font-size: 14px; /* [cite: 126] */
    height: 28px; /* [cite: 127] */
  }

  /* Excel add row */
  .excel-add-row {
    cursor: pointer; /* [cite: 127] */
  }

  .excel-add-row:hover {
    background-color: #f0f8ff; /* [cite: 127] */
  }

  /* Excel add cell */
  .excel-add-cell {
    text-align: left; /* [cite: 128] */ /* Changed from center */
    padding: 8px 8px; /* [cite: 128] */ /* Adjusted padding */
    border-top: 1px solid #d0d0d0; /* Add border */
  }

  /* Excel add button */
  .excel-add-button {
    color: #217346; /* [cite: 129] */
    font-weight: 500; /* [cite: 129] */
  }

  /* Excel column resizer */
  .excel-resizer {
    width: 5px; /* [cite: 129] */
    height: 100%; /* [cite: 130] */
    cursor: col-resize; /* [cite: 130] */
    position: absolute; /* [cite: 130] */
    right: 0; /* [cite: 130] */
    top: 0; /* [cite: 130] */
    background-color: transparent; /* [cite: 130] */
    z-index: 10; /* Ensure above cell content */
  }

  .excel-resizer:hover {
    background-color: #93c5fd; /* [cite: 130] */ /* Lighter blue */
  }

  .excel-resizer-active {
    background-color: #3b82f6 !important; /* [cite: 131] */ /* Brighter blue */
  }

  /* Excel select */
  .excel-select {
    font-size: 14px; /* [cite: 131] */
    width: 100%; /* Ensure select fills space */
  }
  /* Style trigger specifically */
  .excel-select > button {
    height: 100%;
    border-radius: 0;
    border: none; /* Remove trigger border in view mode */
    padding: 6px 8px;
    box-sizing: border-box;
    justify-content: space-between; /* Align icon right */
    background-color: transparent;
  }
  .excel-cell-selected .excel-select > button {
    border: 2px solid #217346; /* Add border only when selected */
    background-color: white;
  }

  /* Excel pagination */
  .excel-pagination {
    margin-top: 16px; /* [cite: 133] */
    padding: 8px; /* [cite: 133] */
    border-top: 1px solid #e0e0e0; /* [cite: 133] */
    display: flex; /* [cite: 133] */
    justify-content: center; /* [cite: 133] */
  }

  .excel-pagination-button {
    color: #217346; /* [cite: 134] */
    /* Add other button styling if needed */
  }

  /* Excel footer */
  .excel-footer {
    display: flex; /* [cite: 134] */
    align-items: center; /* [cite: 134] */
    gap: 16px; /* [cite: 134] */ /* Increased gap */
    padding: 8px 16px; /* [cite: 134] */
    border-top: 1px solid #e0e0e0; /* [cite: 135] */
    background-color: #f9f9f9; /* [cite: 135] */
  }

  .excel-page-size-label {
    font-size: 14px; /* [cite: 135] */
    color: #666666; /* [cite: 135] */
  }

  .excel-page-size-select {
    font-size: 14px; /* [cite: 135] */
  }

  .excel-status-info {
    margin-left: auto; /* [cite: 136] */ /* Push to right */
    font-size: 14px; /* [cite: 136] */
    color: #666666; /* [cite: 136] */
  }

  .error-column-header {
    background-color: #fee2e2 !important; /* [cite: 136] */
    border: 1px solid #ef4444 !important; /* [cite: 136] */ /* Changed border width */
  }

  .error-column-cell {
    /* Applied to TD in add row */
    background-color: #fef2f2 !important; /* [cite: 137] */
    /* Removed border here, applied to input instead */
    /* animation: error-flash 5s; */ /* [cite: 137] */ /* Removed animation */
  }
  .excel-new-row .error-column-cell .excel-input {
    /* Target input in error cell */
    border: 1px solid #ef4444 !important;
    outline: 1px solid #ef4444 !important;
  }

  .excel-new-row-error {
    /* Applied to the TR for adding */
    background-color: #fee2e2 !important; /* [cite: 138] */
    /* animation: error-flash 5s; */ /* [cite: 138] */ /* Removed animation */
  }
  /* Flash effect was removed as it might be jarring */
  /* @keyframes error-flash { ... } */ /**/

  /* Sticky Header Adjustments */
  /* Ensure data headers stay below the numbering column header */
  .excel-column-header {
    position: sticky; /* [cite: 138] */
    top: 28px; /* [cite: 139] */ /* Height of alpha header */
    z-index: 3; /* [cite: 139] */
  }
  /* Adjust z-index for the column letters header */
  .excel-header-row th {
    /* Target TH in alpha row */
    position: sticky;
    top: 0;
    z-index: 5; /* [cite: 139] */
  }
  /* Override z-index for specific sticky headers in alpha row */
  .excel-header-row .excel-column-checkbox-selector {
    z-index: 6;
    background-color: #e6e6e6;
  }
  .excel-header-row .excel-column-checkbox {
    z-index: 5;
    background-color: #e6e6e6;
  } /* Row number in alpha */
  .excel-header-row .excel-actions-header {
    z-index: 5;
    background-color: #e6e6e6;
  }

  /* Add green scrollbar to the table container */
  .table-scroll-container::-webkit-scrollbar {
    height: 12px; /* [cite: 141] */
    width: 12px; /* Added width */
    background-color: #f0fdf4; /* [cite: 141] */
  }

  .table-scroll-container::-webkit-scrollbar-track {
    background: #f0fdf4; /* [cite: 142] */
    border-radius: 6px; /* [cite: 142] */
  }

  .table-scroll-container::-webkit-scrollbar-thumb {
    background: #16a34a; /* [cite: 142] */
    border-radius: 6px; /* [cite: 142] */
    border: 2px solid #f0fdf4; /* [cite: 142] */
  }

  .table-scroll-container::-webkit-scrollbar-thumb:hover {
    background: #22c55e; /* [cite: 143] */
  }

  /* Ensure sticky backgrounds cover content */
  th,
  td {
    background-clip: padding-box; /* Prevents background from going under border */
  }
</style>
