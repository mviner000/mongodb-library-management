<!-- src/components/MongoDBDataTable.vue -->
<script setup lang="ts">
  // ==========================================================================
  // Imports
  // ==========================================================================
  import {
    ref,
    computed,
    watch,
    onMounted,
    Ref,
    inject,
    nextTick,
    onBeforeUnmount,
    reactive,
  } from 'vue' // [cite: 1]
  import { useRoute, useRouter } from 'vue-router' // [cite: 1]
  import { storeToRefs } from 'pinia' // [cite: 1]
  import { useDebounceFn } from '@vueuse/core' // [cite: 1]

  // Store Imports
  import { useDataTableStore } from '@/store/dataTableStore' // [cite: 1]
  import { useUserStore } from '@/store/useUserStore' // [cite: 4]

  // UI Component Imports
  import { Cross2Icon, ReloadIcon, PlusCircledIcon } from '@radix-icons/vue' // [cite: 1]
  import { Button } from '@/components/ui/button' // [cite: 1]
  import { Input } from '@/components/ui/input' // [cite: 1]
  import { TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table' // [cite: 2]
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select' // [cite: 2]
  import {
    Pagination,
    PaginationList,
    PaginationListItem,
    PaginationFirst,
    PaginationLast,
    PaginationNext,
    PaginationPrev,
  } from '@/components/ui/pagination' // [cite: 2]
  import {
    NavigationMenu,
    NavigationMenuContent,
    NavigationMenuItem,
    NavigationMenuList,
    NavigationMenuTrigger,
  } from './ui/navigation-menu' // [cite: 3]
  import { ScrollArea } from '@/components/ui/scroll-area' // [cite: 3]
  import Switch from './ui/switch/Switch.vue' // [cite: 4]
  import Label from './ui/label/Label.vue' // [cite: 4]

  // Custom Component Imports
  import ExcelCellReference from './ExcelCellReference.vue' // [cite: 4]
  import TableActions from './mongodbtable/TableActions.vue' // [cite: 4]
  import StickyTableActions from './mongodbtable/StickyTableActions.vue' // [cite: 4]
  import MongoDBDataTableNavbar from './MongoDBDataTableNavbar.vue' // [cite: 4]
  import StickyLeftSidebar from './StickyLeftSidebar.vue' // [cite: 4]
  import CSVCellReference from './CSVCellReference.vue'

  // ==========================================================================
  // Store Setup
  // ==========================================================================
  const dataTableStore = useDataTableStore() // [cite: 4]
  const userStore = useUserStore() // [cite: 7]

  // Destructured State & Getters (Reactive Refs)
  const { user } = storeToRefs(userStore) // [cite: 7]
  const {
    collectionName,
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
    tableHeaders,
    columnWidths,
    allSelected,
    hiddenColumns,
    visibleHeaders,
    previewMode,
  } = storeToRefs(dataTableStore) // [cite: 4, 5]

  // Destructured Actions (Functions)
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
    updateUIMetadata,
    toggleColumnVisibility,
    updateDocumentField, // Added [cite: 55]
    pinDocument, // Added [cite: 43]
    unpinDocument, // Added [cite: 43]
    // fetchSchema, // Optional direct access
  } = dataTableStore // [cite: 5, 6, 7, 43, 47, 55, 61]

  // ==========================================================================
  // Props
  // ==========================================================================
  const props = defineProps<{
    selectedCollection?: string
    name?: string
    previewData?: any[]
  }>() // [cite: 11]

  // ==========================================================================
  // Local Component State
  // ==========================================================================
  const route = useRoute() // [cite: 7]
  const router = useRouter() // [cite: 7]
  const isSplit = inject<Ref<boolean>>('isSplit')! // [cite: 7]
  const scrollContainer = ref<HTMLElement | null>(null) // [cite: 8]
  const isSidebarOpen = ref(false) // [cite: 9]
  const searchQuery = ref<Record<string, string>>({}) // [cite: 9]
  const selectedCell = ref<{ colIndex: number; rowNumber: number } | null>(null) // [cite: 10]
  const timeoutId = ref<number | null>(null) // For error message timeout [cite: 11]
  const showPinTooltip = ref(false) // [cite: 11]
  const highlightedColumn = ref<string | null>(null) // [cite: 62]
  const showContextMenu = ref(false) // [cite: 40]
  const contextMenuPosition = ref({ x: 0, y: 0 }) // [cite: 40]
  const selectedCellInfo = ref<{ rowIndex: number; header: string } | null>(null) // [cite: 41]
  const highlightedDocumentId = ref<string | null>(null) // [cite: 49]
  let highlightTimeout: ReturnType<typeof setTimeout> | null = null // [cite: 49]
  const fileInput = ref<HTMLInputElement | null>(null) // [cite: 71]
  const editingHeader = ref<string | null>(null) // [cite: 59]
  const editedShortName = ref('') // [cite: 59]

  // Resizing States
  const resizingState = ref({
    isResizing: false,
    header: '',
    startX: 0,
    startWidth: 0,
    currentWidth: 0,
  }) // [cite: 9]
  const alphaResizingState = ref({
    isResizing: false,
    columnIndex: -1,
    startX: 0,
    startWidth: 0,
    currentWidth: 0,
  }) // [cite: 9]
  const rowResizingState = ref({
    isResizing: false,
    documentId: '',
    startY: 0,
    startHeight: 40, // Default row height
    currentHeight: 40,
  }) // [cite: 51, 52]

  // Drag State
  const dragState = reactive({
    isDragging: false,
    draggedIndex: -1,
    targetIndex: -1,
  }) // [cite: 65]

  // ==========================================================================
  // Computed Properties
  // ==========================================================================
  const documents = computed(() => {
    return props.previewData || dataTableStore.documents // [cite: 69, 70]
  })

  const paginatedDocuments = computed(() => {
    if (props.previewData) {
      // Preview mode: Show all data without pagination
      return documents.value // [cite: 72]
    } else {
      // Regular mode: Use store pagination
      const start = (currentPage.value - 1) * pageSize.value // [cite: 72]
      const end = start + pageSize.value // [cite: 72]
      return documents.value.slice(start, end) // [cite: 72]
    }
  })

  const columnLetters = computed(() => {
    return visibleHeaders.value.map((_, index) => getColumnLabel(index)) // [cite: 12]
  })

  const numberColumnWidth = computed(() => {
    const maxDigits = documents.value.length > 0 ? String(documents.value.length).length : 1 // [cite: 12]
    return `${Math.max(3, maxDigits + 1)}ch` // [cite: 12]
  })

  const totalTableWidth = computed(() => {
    // Calculate the total width of visible data columns
    const dataColumnsWidth = visibleHeaders.value.reduce(
      (acc, header) => acc + (columnWidths.value[header] || 200), // Default width if not specified [cite: 13, 14]
      0
    )

    const selectColWidth = 40 // Width for the selection column [cite: 14]
    const rowNumColWidth = 30 // Width for the row number column [cite: 14]
    const actionsColWidth = 60 // Width for the actions column [cite: 14]

    // Return the total width including only visible columns
    return selectColWidth + rowNumColWidth + dataColumnsWidth + actionsColWidth + 1 // +1 for border [cite: 14]
  })

  const isSelectedArchived = computed(() => {
    if (!selectedCellInfo.value) {
      return false // [cite: 45]
    }

    const rowIndex = selectedCellInfo.value.rowIndex // [cite: 45]
    const document = paginatedDocuments.value[rowIndex] // [cite: 45]
    const isArchived = document?.is_archive === true // [cite: 46]

    return isArchived // [cite: 46]
  })

  const selectedDocumentIsPinned = computed(() => {
    if (!selectedCellInfo.value) return false // [cite: 46]
    if (!user.value) return false // Add this check for null user [cite: 46]

    const doc = paginatedDocuments.value[selectedCellInfo.value.rowIndex] // [cite: 46]
    return doc?.pinned_by?.includes(user.value.id) // [cite: 46]
  })

  const pinnedDocuments = computed(() => {
    if (!user.value) return [] // [cite: 47]
    return documents.value.filter((doc) => doc.pinned_by?.includes(user.value?.id)) // [cite: 47]
  })

  const previewRowHeights = computed(() => {
    if (!previewMode.value || !collectionName.value) return {} // [cite: 55, 56]
    const heights = sessionStorage.getItem(`previewRowHeights-${collectionName.value}`) // [cite: 56]
    return heights ? JSON.parse(heights) : {} // [cite: 56, 57]
  })

  const isPreviewMode = computed(() => previewMode.value) // [cite: 70]
  const showImportInPreview = computed(() => isPreviewMode.value && props.previewData) // [cite: 70]
  const isImportRoute = computed(() => route.path.includes('/import-csv')) // [cite: 71]

  // ==========================================================================
  // Watchers
  // ==========================================================================

  // Watch route parameter 'name'
  watch(
    () => route.params.name,
    (newName) => {
      if (previewMode.value) return // Skip in preview [cite: 18]
      const nameStr = Array.isArray(newName) ? newName[0] : newName // [cite: 18]
      if (nameStr && nameStr !== collectionName.value) {
        setCollection(nameStr) // [cite: 18]
      }
    },
    { immediate: true } // [cite: 19]
  )

  // Watch the prop 'selectedCollection'
  watch(
    () => props.selectedCollection,
    (newVal) => {
      if (newVal && newVal !== collectionName.value) {
        setCollection(newVal) // [cite: 19]
        if (route.params.name !== newVal) {
          router.push(`/collection/${newVal}`) // [cite: 19]
        }
      }
    }
  )

  // Watch the store's collectionName to sync the route if needed
  watch(collectionName, (newName, oldName) => {
    if (newName && newName !== oldName && route.params.name !== newName) {
      router.push(`/collection/${newName}`) // [cite: 20]
    }
  })

  // Watch for error message to auto-clear
  watch(errorMessage, (newVal) => {
    if (newVal) {
      if (timeoutId.value) {
        clearTimeout(timeoutId.value) // [cite: 20]
      }
      timeoutId.value = setTimeout(() => {
        clearError() // Use store action [cite: 21]
        timeoutId.value = null // [cite: 21]
      }, 2500) as unknown as number // [cite: 21]
    }
  })

  // Watch isAdding to prefetch reference options
  watch(isAdding, async (newVal) => {
    if (newVal) {
      tableHeaders.value.forEach((field) => {
        // Use tableHeaders getter [cite: 21]
        const refCollection = getReferencedCollection(field) // [cite: 21]
        if (refCollection && !referenceOptions.value[refCollection]) {
          // [cite: 22]
          fetchReferenceOptions(refCollection) // Use store action [cite: 22]
        }
      })
    }
  }) // [cite: 22]

  // Watch pinnedDocuments (removed console.log)
  const debouncedLogPinnedDocuments = useDebounceFn((newVal) => {
    // Empty function now that console.log is removed [cite: 47]
  }, 500)

  watch(pinnedDocuments, (newVal) => debouncedLogPinnedDocuments(newVal), {
    immediate: true, // [cite: 48]
    deep: true, // [cite: 48]
  })

  // Watch previewMode (removed console.log)
  watch(
    previewMode,
    (newVal, oldVal) => {
      // All console.log statements removed [cite: 69]
    },
    { immediate: true } // [cite: 69]
  )

  // Watch route to set preview mode
  watch(
    isImportRoute,
    (newVal) => {
      previewMode.value = newVal // [cite: 71]
    },
    { immediate: true } // [cite: 71]
  )

  // ==========================================================================
  // Lifecycle Hooks
  // ==========================================================================
  onMounted(async () => {
    if (previewMode.value) return // Skip fetching in preview mode [cite: 22]

    await fetchCollections() // [cite: 22]
    const routeName = Array.isArray(route.params.name) ? route.params.name[0] : route.params.name // [cite: 23]
    const initialCollection = routeName || props.selectedCollection // [cite: 24]

    if (initialCollection && initialCollection !== collectionName.value) {
      await setCollection(initialCollection) // [cite: 24]
    } else if (collectionName.value && documents.value.length === 0 && !isLoading.value) {
      try {
        await dataTableStore.fetchSchema() // [cite: 24]
        await dataTableStore.fetchDocuments() // [cite: 24]
      } catch (error) {
        // Handle error [cite: 24]
      }
    }

    // Event listeners
    window.addEventListener('click', closeContextMenu) // [cite: 45]
    document.addEventListener('click', checkClickOutsideHeaders) // [cite: 45]
  })

  onBeforeUnmount(() => {
    // Cleanup listeners
    document.removeEventListener('click', checkClickOutsideHeaders) // [cite: 45]
    window.removeEventListener('click', closeContextMenu) // [cite: 45]

    // Clear any pending timeouts
    if (timeoutId.value) {
      clearTimeout(timeoutId.value)
    }
    if (highlightTimeout) {
      clearTimeout(highlightTimeout)
    }
  })

  // ==========================================================================
  // Utility Functions
  // ==========================================================================
  const getColumnLabel = (index: number): string => {
    let label = '' // [cite: 11]
    let i = index // [cite: 11]
    do {
      const remainder = i % 26 // [cite: 11]
      label = String.fromCharCode(65 + remainder) + label // [cite: 12]
      i = Math.floor(i / 26) - 1 // [cite: 12]
    } while (i >= 0)
    return label // [cite: 12]
  } // [cite: 12]

  const isFieldRequired = (field: string): boolean => {
    return collectionSchema.value.required?.includes(field) || false // [cite: 15]
  } // [cite: 15]

  const formatSchemaValue = (value: any, bsonType?: string | string[]) => {
    if (value === undefined || value === null) return '' // [cite: 15]
    const type = bsonType ? (Array.isArray(bsonType) ? bsonType[0] : bsonType) : typeof value // [cite: 15, 16]

    if (type === 'date' && value instanceof Date) {
      // Check if it's already a Date object [cite: 16]
      return value.toLocaleString() // [cite: 16]
    } else if (type === 'date') {
      // Try parsing if it's not a Date object [cite: 16]
      try {
        return new Date(value).toLocaleString() // [cite: 16]
      } catch {
        return String(value) // Fallback [cite: 16]
      }
    }
    if (typeof value === 'object') {
      return JSON.stringify(value, null, 2) // Keep original formatting [cite: 17]
    }
    return String(value) // [cite: 17]
  } // [cite: 17]

  const filteredOptions = (field: string) => {
    const refCollection = getReferencedCollection(field) // [cite: 17]
    if (!refCollection) return [] // [cite: 17]
    const options = referenceOptions.value[refCollection] || [] // [cite: 17, 18]
    const query = (searchQuery.value[field] || '').toLowerCase() // [cite: 18]
    return options.filter((opt) => opt.label.toLowerCase().includes(query)) // [cite: 18]
  } // [cite: 18]

  const shortName = (header: string) => {
    return collectionSchema.value.ui?.short_names?.[header] || header // [cite: 59, 60]
  }

  function getActualIndex(visibleIndex: number) {
    let actualIndex = 0 // [cite: 65]
    let visibleCount = 0 // [cite: 65]

    while (visibleCount <= visibleIndex && actualIndex < tableHeaders.value.length) {
      // [cite: 65]
      if (!hiddenColumns.value.includes(tableHeaders.value[actualIndex])) {
        // [cite: 65]
        visibleCount++ // [cite: 65]
      }
      actualIndex++ // [cite: 65]
    }

    return actualIndex - 1 // [cite: 66]
  }

  // ==========================================================================
  // Debounced Functions
  // ==========================================================================
  const debouncedSaveWidths = useDebounceFn(async () => {
    await saveColumnWidthsToBackend() // Call store action [cite: 25]
  }, 750) // Keep debounce consistent [cite: 25]

  const debouncedRowHeightSave = useDebounceFn(async (documentId: string, height: number) => {
    try {
      // [cite: 54]
      await updateDocumentField(documentId, 'row_height', height) // [cite: 55]
    } catch (error) {
      // Error handling without console.error [cite: 55]
    }
  }, 500) // 500ms delay [cite: 55]

  // ==========================================================================
  // Methods
  // ==========================================================================

  // --- Error Handling ---
  const closeErrorManual = () => {
    if (timeoutId.value) {
      clearTimeout(timeoutId.value) // [cite: 25]
      timeoutId.value = null // [cite: 25]
    }
    clearError() // Use store action [cite: 25]
  } // Matches original @click="closeError" [cite: 25]

  // --- Column Resizing ---
  const startAlphaResize = (columnIndex: number, event: MouseEvent) => {
    const header = tableHeaders.value[columnIndex] // Use getter [cite: 26]
    if (!header) return // [cite: 26]
    const currentWidth = columnWidths.value[header] || 200 // Use getter [cite: 26, 27]

    alphaResizingState.value = {
      isResizing: true,
      columnIndex,
      startX: event.clientX,
      startWidth: currentWidth,
      currentWidth: currentWidth,
    } // [cite: 27]
    document.addEventListener('mousemove', handleAlphaMouseMove) // [cite: 27]
    document.addEventListener('mouseup', stopAlphaResize) // [cite: 27]
    event.preventDefault() // [cite: 27]
  } // [cite: 27]

  const handleAlphaMouseMove = (event: MouseEvent) => {
    if (!alphaResizingState.value.isResizing) return // [cite: 27]
    const delta = event.clientX - alphaResizingState.value.startX // [cite: 27]
    const newWidth = Math.max(50, alphaResizingState.value.startWidth + delta) // [cite: 27]
    alphaResizingState.value.currentWidth = newWidth // [cite: 28]

    const header = tableHeaders.value[alphaResizingState.value.columnIndex] // [cite: 28]
    if (header && collectionSchema.value.ui) {
      // Temporarily update local schema for visual feedback [cite: 28]
      collectionSchema.value.ui.columnWidths = {
        ...collectionSchema.value.ui.columnWidths,
        [header]: newWidth,
      } // [cite: 28]
    }
  }

  const stopAlphaResize = async () => {
    if (!alphaResizingState.value.isResizing) return // [cite: 28]
    const header = tableHeaders.value[alphaResizingState.value.columnIndex] // [cite: 28]
    const finalWidth = alphaResizingState.value.currentWidth // [cite: 28]
    alphaResizingState.value.isResizing = false // [cite: 29]
    document.removeEventListener('mousemove', handleAlphaMouseMove) // [cite: 29]
    document.removeEventListener('mouseup', stopAlphaResize) // [cite: 29]

    if (header) {
      await updateColumnWidth(header, finalWidth) // Update store [cite: 29]
      debouncedSaveWidths() // Trigger debounced save [cite: 29]
    }
  }

  const resetAlphaColumnWidth = async (columnIndex: number) => {
    const header = tableHeaders.value[columnIndex] // [cite: 29]
    if (header) {
      await resetColumnWidth(header) // Update store [cite: 29]
      debouncedSaveWidths() // Trigger save [cite: 29]
    }
  } // [cite: 29]

  const startResize = (header: string, event: MouseEvent) => {
    const currentWidth = columnWidths.value[header] || 200 // [cite: 30, 31]
    resizingState.value = {
      isResizing: true,
      header,
      startX: event.clientX,
      startWidth: currentWidth,
      currentWidth: currentWidth,
    } // [cite: 31]
    document.addEventListener('mousemove', handleMouseMove) // [cite: 31]
    document.addEventListener('mouseup', stopResize) // [cite: 31]
    event.preventDefault() // [cite: 31]
  } // [cite: 31]

  const handleMouseMove = (event: MouseEvent) => {
    if (!resizingState.value.isResizing) return // [cite: 31]
    const delta = event.clientX - resizingState.value.startX // [cite: 31]
    const newWidth = Math.max(50, resizingState.value.startWidth + delta) // [cite: 31]
    resizingState.value.currentWidth = newWidth // [cite: 32]

    const header = resizingState.value.header // [cite: 32]
    if (header && collectionSchema.value.ui) {
      // Temporarily update local schema for visual feedback [cite: 32]
      collectionSchema.value.ui.columnWidths = {
        ...collectionSchema.value.ui.columnWidths,
        [header]: newWidth,
      } // [cite: 32]
    }
  }

  const stopResize = async () => {
    if (!resizingState.value.isResizing) return // [cite: 32]
    const header = resizingState.value.header // [cite: 32]
    const finalWidth = resizingState.value.currentWidth // [cite: 32]
    resizingState.value.isResizing = false // [cite: 33]
    document.removeEventListener('mousemove', handleMouseMove) // [cite: 33]
    document.removeEventListener('mouseup', stopResize) // [cite: 33]

    if (header) {
      await updateColumnWidth(header, finalWidth) // Update store [cite: 33]
      debouncedSaveWidths() // Trigger save [cite: 33]
    }
  }

  // Commented out original reset function as it might clash with store action name
  // const resetColumnWidth = async (header: string) => { // Renamed from original resetDataColumnWidth for clarity
  //   await resetColumnWidth(header); // Call store action (make sure names don't clash) [cite: 33]
  //   debouncedSaveWidths(); // [cite: 34]
  // }; // [cite: 35]

  // --- Row Resizing ---
  const startRowResize = (documentId: string, event: MouseEvent) => {
    const doc = documents.value.find((d) => d._id.$oid === documentId) // [cite: 52]
    if (!doc) return // [cite: 52]

    const currentHeight = doc.row_height || 40 // [cite: 52, 53]
    rowResizingState.value = {
      isResizing: true,
      documentId,
      startY: event.clientY,
      startHeight: currentHeight,
      currentHeight,
    } // [cite: 53]

    document.addEventListener('mousemove', handleRowMouseMove) // [cite: 53]
    document.addEventListener('mouseup', stopRowResize) // [cite: 53]
  }

  const handleRowMouseMove = (event: MouseEvent) => {
    if (!rowResizingState.value.isResizing) return // [cite: 53]

    const delta = event.clientY - rowResizingState.value.startY // [cite: 53]
    const newHeight = Math.max(40, rowResizingState.value.startHeight + delta) // [cite: 53]

    rowResizingState.value.currentHeight = newHeight // [cite: 53]

    // Update local document for visual feedback
    const docIndex = documents.value.findIndex(
      (d) => d._id.$oid === rowResizingState.value.documentId // [cite: 54]
    )
    if (docIndex !== -1) {
      const updatedDoc = {
        ...documents.value[docIndex],
        row_height: newHeight,
      } // [cite: 54]
      documents.value.splice(docIndex, 1, updatedDoc) // [cite: 54]
    }
  }

  const stopRowResize = async () => {
    if (!rowResizingState.value.isResizing) return // [cite: 57]
    const { documentId, currentHeight } = rowResizingState.value // [cite: 57]

    rowResizingState.value.isResizing = false // [cite: 57]
    document.removeEventListener('mousemove', handleRowMouseMove) // [cite: 57]
    document.removeEventListener('mouseup', stopRowResize) // [cite: 57]

    if (previewMode.value) {
      const newHeights = {
        ...previewRowHeights.value,
        [documentId]: currentHeight,
      } // [cite: 57]
      sessionStorage.setItem(
        `previewRowHeights-${collectionName.value}`,
        JSON.stringify(newHeights) // [cite: 58]
      )
    } else {
      debouncedRowHeightSave(documentId, currentHeight) // [cite: 58]
    }
  }

  // --- Cell Click/Edit Handling ---
  const handleCellClick = (rowIndex: number, header: string, value: any) => {
    // Value param from original template [cite: 35]
    if (isSaving.value || ['_id', 'created_at', 'updated_at'].includes(header)) return // [cite: 35]

    // Use the original value passed from the template click event
    startEditingCell(rowIndex, header, value) // Use store action [cite: 35]

    // Update selectedCell for visual feedback (Excel-like)
    const actualRowNumber = (currentPage.value - 1) * pageSize.value + rowIndex + 1 // [cite: 35]
    const colIndex = tableHeaders.value.indexOf(header) // [cite: 35]
    selectedCell.value = { colIndex, rowNumber: actualRowNumber } // [cite: 36]

    // Focus logic
    nextTick(() => {
      // Simplified focus selector - adjust if needed [cite: 36]
      const inputElement = scrollContainer.value?.querySelector<
        HTMLInputElement | HTMLTextAreaElement
      >(
        `tr:nth-child(${rowIndex + 1}) td[class*='excel-cell'] textarea, tr:nth-child(${rowIndex + 1}) td[class*='excel-cell'] input` // [cite: 36]
      )
      inputElement?.focus() // [cite: 36]
      if (inputElement && (inputElement.tagName === 'TEXTAREA' || inputElement.type === 'text')) {
        // [cite: 37]
        inputElement.select() // [cite: 37]
      }
    })
  } // Combines original template call [cite: 37] with store logic

  const handleEditBlur = async () => {
    setTimeout(async () => {
      const activeElement = document.activeElement // [cite: 37]
      // Basic check if focus is still within the editing area (might need refinement) [cite: 37]
      const isStillEditing =
        editingCell.value && // [cite: 38]
        activeElement && // [cite: 38]
        (activeElement.closest('.excel-cell-selected') || // [cite: 38]
          activeElement.closest('[data-radix-popper-content-wrapper]')) // Consider Radix poppers for Select [cite: 38]

      if (editingCell.value && !isStillEditing) {
        await saveEdit() // Use store action [cite: 38]
      } else if (!isStillEditing) {
        cancelEdit() // Use store action [cite: 38]
      }
    }, 100) // [cite: 39]
  } // Connects to @blur event [cite: 39]

  // --- Header Editing ---
  const startEditingHeader = (header: string) => {
    editingHeader.value = header // [cite: 60]
    editedShortName.value = shortName(header) // [cite: 60]
  }

  const isEditingHeader = (header: string) => {
    return editingHeader.value === header // [cite: 60]
  }

  const saveShortName = async (header: string) => {
    if (!editedShortName.value.trim()) return // [cite: 60]

    const newShortNames = {
      ...collectionSchema.value.ui?.short_names,
      [header]: editedShortName.value,
    } // [cite: 60]

    try {
      await updateUIMetadata({ short_names: newShortNames }) // [cite: 60]
      editingHeader.value = null // [cite: 61]
    } catch (error) {
      // Handle error [cite: 61]
    }
  }

  const cancelEditTextHead = () => {
    editingHeader.value = null // [cite: 61]
  }

  // --- Column Highlighting ---
  const handleColumnHighlight = (index: number) => {
    // Count hidden columns before the visible index to get the actual index in tableHeaders [cite: 62]
    let hiddenColumnsCount = 0 // [cite: 62]
    for (let i = 0; i <= index; i++) {
      const visibleIndex = i + hiddenColumnsCount // [cite: 62]
      if (
        visibleIndex < tableHeaders.value.length && // [cite: 62]
        hiddenColumns.value.includes(tableHeaders.value[visibleIndex]) // [cite: 62]
      ) {
        hiddenColumnsCount++ // [cite: 63]
        i-- // Stay at the same visible index since we found a hidden column [cite: 63]
      }
    }

    // Adjust index to account for hidden columns
    const adjustedIndex = index + hiddenColumnsCount // [cite: 63]
    const header = tableHeaders.value[adjustedIndex] // [cite: 63]

    // Clear both selected cell and editing state
    selectedCell.value = null // [cite: 63]
    cancelEdit() // This clears editingCell in the store [cite: 63]

    if (selectedRows.value.size > 0) {
      const confirmed = confirm('You have selected rows. Unselect them to highlight the column?') // [cite: 64]

      if (confirmed) {
        resetSelection() // [cite: 64]
        highlightedColumn.value = header // [cite: 64]
      }
    } else {
      if (header === highlightedColumn.value) {
        highlightedColumn.value = null // [cite: 64]
      } else {
        highlightedColumn.value = header // [cite: 64]
      }
    } // [cite: 65]
  }

  const checkClickOutsideHeaders = (event: MouseEvent) => {
    const target = event.target as HTMLElement // [cite: 44]
    const isHeaderClick = target.closest('.excel-column-letter, .excel-column-header') // [cite: 44]
    if (!isHeaderClick) {
      highlightedColumn.value = null // [cite: 45]
    }
  }

  // --- Drag and Drop Columns ---
  function onDragStart(event: DragEvent, visibleIndex: number) {
    const actualIndex = getActualIndex(visibleIndex) // [cite: 66]
    dragState.isDragging = true // [cite: 66]
    dragState.draggedIndex = actualIndex // [cite: 66]
    event.dataTransfer?.setData('text/plain', '') // [cite: 66]
    highlightedColumn.value = tableHeaders.value[actualIndex] // [cite: 66]
  }

  function onDragOver(event: DragEvent, visibleIndex: number) {
    if (!dragState.isDragging) return // [cite: 66]
    event.preventDefault() // [cite: 66]

    const actualIndex = getActualIndex(visibleIndex) // [cite: 66]
    if (dragState.targetIndex !== actualIndex) {
      dragState.targetIndex = actualIndex // [cite: 66]
    }
  }

  function onDrop(event: DragEvent, visibleIndex: number) {
    event.preventDefault() // [cite: 66]
    if (!dragState.isDragging) return // [cite: 67]

    const targetActualIndex = getActualIndex(visibleIndex) // [cite: 67]
    const draggedIndex = dragState.draggedIndex // [cite: 67]

    // Existing column order logic
    const currentUI = collectionSchema.value.ui || {} // [cite: 67, 68]
    let columnOrder = currentUI.columnOrder ? [...currentUI.columnOrder] : [...tableHeaders.value] // [cite: 68]
    const allHeaders = Object.keys(collectionSchema.value.properties || {}) // [cite: 68]

    // Clean and reorder
    columnOrder = columnOrder.filter((h) => allHeaders.includes(h)) // [cite: 68]
    const missing = allHeaders.filter((h) => !columnOrder.includes(h)) // [cite: 68]
    columnOrder.push(...missing) // [cite: 68]

    // Perform the move using actual indices
    const [moved] = columnOrder.splice(draggedIndex, 1) // [cite: 68]
    columnOrder.splice(targetActualIndex, 0, moved) // [cite: 68]

    // Update UI metadata
    updateUIMetadata({ columnOrder }) // [cite: 68]

    // Reset state
    dragState.isDragging = false // [cite: 68]
    dragState.draggedIndex = -1 // [cite: 69]
    dragState.targetIndex = -1 // [cite: 69]
    highlightedColumn.value = null // [cite: 69]
  }

  function onDragEnd() {
    dragState.isDragging = false // [cite: 69]
    dragState.draggedIndex = -1 // [cite: 69]
    dragState.targetIndex = -1 // [cite: 69]
  }

  // --- Context Menu ---
  const handleRightClick = (rowIndex: number, header: string, event: MouseEvent) => {
    event.preventDefault() // [cite: 41]
    selectedCellInfo.value = { rowIndex, header } // [cite: 41]

    // Calculate actual row number and column index for highlighting
    const actualRowNumber = (currentPage.value - 1) * pageSize.value + rowIndex + 1 // [cite: 41]
    const colIndex = tableHeaders.value.indexOf(header) // [cite: 41]
    selectedCell.value = { colIndex, rowNumber: actualRowNumber } // [cite: 41]

    // Position context menu
    const offset = 55 // [cite: 41]
    contextMenuPosition.value = {
      x: event.clientX, // [cite: 42]
      y: event.clientY - offset, // [cite: 42]
    }
    showContextMenu.value = true // [cite: 42]
  }

  const closeContextMenu = () => {
    showContextMenu.value = false // [cite: 42]
    selectedCellInfo.value = null // [cite: 42]
  }

  const pinCell = async () => {
    if (!selectedCellInfo.value) {
      return // [cite: 42]
    }
    if (isSelectedArchived.value) {
      return // [cite: 42]
    }
    // Add check for null user
    if (!user.value) {
      // [cite: 43]
      return // [cite: 43]
    }
    const rowIndex = selectedCellInfo.value.rowIndex // [cite: 43]
    const doc = paginatedDocuments.value[rowIndex] // [cite: 43]
    if (!doc) {
      return // [cite: 43]
    }
    const isPinned = doc.pinned_by?.includes(user.value.id) // [cite: 43]
    try {
      if (isPinned) {
        await unpinDocument(doc._id.$oid) // [cite: 43]
      } else {
        await pinDocument(doc._id.$oid) // [cite: 43]
      }

      // Add this critical line to refresh documents after pin/unpin operation
      await fetchDocuments() // [cite: 44]
    } catch (error) {
      // Error handling without console.error [cite: 44]
    }
    closeContextMenu() // [cite: 44]
  }

  const bookmarkCell = () => {
    if (!selectedCellInfo.value) return // [cite: 44]
    closeContextMenu() // [cite: 44]
  }

  const togglePinStatus = async (docId: string, currentPinStatus: boolean): Promise<void> => {
    try {
      if (currentPinStatus) {
        await unpinDocument(docId) // [cite: 46]
      } else {
        await pinDocument(docId) // [cite: 47]
      }
    } catch (error) {
      // Error handling without console.error [cite: 47]
    }
  }

  // --- Document Navigation/Highlighting ---
  const handleDocumentNavigation = (docId: string) => {
    // Clear existing timeout if any
    if (highlightTimeout) clearTimeout(highlightTimeout) // [cite: 49]

    // Find document index in full dataset
    const index = documents.value.findIndex((doc) => doc._id.$oid === docId) // [cite: 49]
    if (index === -1) return // [cite: 49]

    // Calculate and set correct page
    const page = Math.ceil((index + 1) / pageSize.value) // [cite: 49]
    setPage(page) // [cite: 49]

    // Set highlight and auto-clear after 2s
    highlightedDocumentId.value = docId // [cite: 49, 50]
    highlightTimeout = setTimeout(() => {
      highlightedDocumentId.value = null // [cite: 50]
    }, 2000) // [cite: 50]

    // Scroll to row after DOM update
    nextTick(() => {
      const row = scrollContainer.value?.querySelector(`[data-document-id="${docId}"]`) // [cite: 50]
      if (row) {
        row.scrollIntoView({
          behavior: 'smooth',
          block: 'nearest',
          inline: 'nearest', // [cite: 50]
        })

        // Add visual pulse effect [cite: 51]
        row.classList.add('highlight-pulse') // [cite: 51]
        setTimeout(() => row.classList.remove('highlight-pulse'), 1000) // [cite: 51]
      }
    })
  }

  // --- Template specific handlers ---
  const handleViewChange = (view: string) => {
    changeView(view) // Use store action [cite: 39]
  } // [cite: 39]

  const onPageChange = (page: number) => {
    setPage(page) // Use store action [cite: 39]
  } // [cite: 39]

  const handleDeleteStart = (id: string) => {
    dataTableStore.pendingDeleteId = id // [cite: 39, 40]
  } // [cite: 40]
  const handleDeleteEnd = () => {
    dataTableStore.pendingDeleteId = null // [cite: 40]
  } // [cite: 40]

  const triggerFileImport = () => {
    fileInput.value?.click() // [cite: 71]
  }

  // --- Emit events ---
  const emit = defineEmits(['preview-data-update']) // [cite: 71]

  // ==========================================================================
  // Expose (if needed by parent components)
  // ==========================================================================
  defineExpose({
    fetchDocuments: dataTableStore.fetchDocuments, // [cite: 40]
    fetchCollections: dataTableStore.fetchCollections, // [cite: 40]
    setCollection: dataTableStore.setCollection, // [cite: 40]
  })
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
      :preview-mode="previewMode"
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
        <CSVCellReference
          :selectedCell="selectedCell"
          :selectedRows="selectedRows"
          :isSidebarOpen="isSidebarOpen"
          :previewMode="false"
          @reset-selection=""
        />
        <ExcelCellReference
          :preview-mode="previewMode"
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
        <!-- Visibility of Columns -->
        <div class="z-40 mt-10 p-2 bg-[#217346]/90 flex gap-2">
          <NavigationMenu>
            <NavigationMenuList>
              <NavigationMenuItem>
                <NavigationMenuTrigger size="sm">Visibility</NavigationMenuTrigger>
                <NavigationMenuContent>
                  <div class="p-2 w-48">
                    <div
                      v-for="header in tableHeaders"
                      :key="header"
                      class="flex items-center space-x-2 py-1 px-2 rounded hover:bg-green-100 transition-colors duration-150 group"
                    >
                      <input
                        type="checkbox"
                        :id="`checkbox-${header}`"
                        :checked="!hiddenColumns.includes(header)"
                        @change="toggleColumnVisibility(header)"
                        class="cursor-pointer accent-green-600"
                      />
                      <label
                        :for="`checkbox-${header}`"
                        class="flex-1 cursor-pointer text-sm group-hover:text-green-800"
                      >
                        {{ collectionSchema.ui?.short_names?.[header] || header }}
                      </label>
                    </div>
                  </div>
                </NavigationMenuContent>
              </NavigationMenuItem>
            </NavigationMenuList>
          </NavigationMenu>

          <div class="flex items-center space-x-2">
            <Switch
              id="preview-live-toggle"
              :modelValue="previewMode"
              :disabled="isImportRoute"
              @update:modelValue="
                (value) => {
                  if (!isImportRoute) previewMode = value
                }
              "
            />
            <Label
              class="text-white cursor-pointer"
              for="preview-live-toggle"
            >
              {{ previewMode ? 'Mode: Preview' : 'Mode: Live' }}
            </Label>
          </div>

          <router-link
            :to="`/collection/${collectionName}/import-csv`"
            class="ml-4"
          >
            <Button
              variant="outline"
              size="sm"
            >
              Link to Import CSV Page
            </Button>
          </router-link>
        </div>

        <table
          class="excel-table"
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
              <!-- TableHead for column letters -->
              <TableHead
                v-for="(letter, index) in columnLetters"
                :key="`letter-${index}`"
                class="select-none excel-column-letter relative"
                :class="{
                  'drop-target': getActualIndex(index) === dragState.targetIndex,
                  'highlighted-column': highlightedColumn === visibleHeaders[index],
                }"
                @click="handleColumnHighlight(index)"
                draggable="true"
                @dragstart="onDragStart($event, index)"
                @dragover.prevent="onDragOver($event, index)"
                @dragend="onDragEnd"
                @drop="onDrop($event, index)"
                :style="{
                  width:
                    alphaResizingState.isResizing && alphaResizingState.columnIndex === index
                      ? `${alphaResizingState.currentWidth}px`
                      : `${columnWidths[tableHeaders[index]] || 200}px`,
                }"
              >
                <div
                  class="flex items-center justify-center"
                  :class="{
                    // UPDATED condition: Compare with the header from visibleHeaders at the current index
                    'cursor-pointer bg-blue-500 text-white py-[9px] px-3 -mx-[10px] -my-[10px]':
                      highlightedColumn === visibleHeaders[index],
                  }"
                >
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
              <!-- TableHead for column headers -->
              <TableHead
                v-for="(header, index) in visibleHeaders"
                :key="header"
                class="cursor-pointer excel-column-header font-bold text-black relative"
                :class="{
                  'error-column-header': header === errorColumn && isAdding,
                  'highlighted-column': highlightedColumn === header,
                  dragging: dragState.draggedIndex === getActualIndex(index),
                  'drop-target': dragState.targetIndex === getActualIndex(index),
                }"
                :style="{
                  width:
                    resizingState.isResizing && resizingState.header === header
                      ? `${resizingState.currentWidth}px`
                      : `${columnWidths[header] || 200}px`,
                }"
                @click="handleColumnHighlight(index)"
                draggable="true"
                @dragstart="onDragStart($event, index)"
                @dragover.prevent="onDragOver($event, index)"
                @dragend="onDragEnd"
                @drop="onDrop($event, index)"
              >
                <div class="flex items-center justify-between">
                  <div
                    class="header-content"
                    @dblclick="startEditingHeader(header)"
                  >
                    <span v-if="!isEditingHeader(header)">
                      <!-- Use short name with fallback to header name -->
                      {{ collectionSchema.ui?.short_names?.[header] || header }}
                      <span
                        v-if="isFieldRequired(header)"
                        class="text-red-500"
                        >*</span
                      >
                    </span>
                    <input
                      v-else
                      v-model="editedShortName"
                      @keyup.enter="saveShortName(header)"
                      @keyup.esc="cancelEditTextHead"
                      @blur="saveShortName(header)"
                      class="header-edit-input"
                    />
                  </div>
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
                :style="{
                  height: previewMode
                    ? `${previewRowHeights[doc._id.$oid] || doc.row_height || 40}px`
                    : `${doc.row_height || 40}px`,
                }"
                class="excel-data-row relative"
                :class="{
                  'highlight-row': highlightedDocumentId === doc._id.$oid,
                  'pending-delete-outline': doc._id.$oid === pendingDeleteId,
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
                  class="excel-row-number relative"
                  :style="{
                    width: numberColumnWidth,
                    minWidth: numberColumnWidth,
                    maxWidth: numberColumnWidth,
                  }"
                >
                  <div
                    class="w-full h-full flex items-center justify-center cursor-pointer relative"
                    @click.stop="
                      !previewMode &&
                      togglePinStatus(doc._id.$oid, user && doc.pinned_by?.includes(user.id))
                    "
                    :class="[
                      { 'hover:bg-gray-100': !doc.is_archive },
                      { 'cursor-default': previewMode },
                    ]"
                    :title="
                      previewMode
                        ? ''
                        : doc.is_archive
                          ? 'Cannot pin/unpin archived items'
                          : user && doc.pinned_by?.includes(user.id)
                            ? 'Click to unpin'
                            : 'Click to pin'
                    "
                  >
                    <span class="">{{ (currentPage - 1) * pageSize + rowIndex + 1 }}</span>

                    <span
                      v-if="user && doc.pinned_by?.includes(user.id)"
                      class="text-xl left-2 bottom-2 absolute z-10"
                      :class="{ hidden: previewMode }"
                    >
                      📌
                    </span>
                  </div>

                  <!-- Row resize handle -->
                  <div
                    class="row-resize-handle absolute bottom-[-1px] left-0 right-0 h-2 cursor-row-resize z-10"
                    @mousedown.prevent="startRowResize(doc._id.$oid, $event)"
                  ></div>
                </TableCell>
                <TableCell
                  v-for="header in visibleHeaders"
                  :key="`${doc._id.$oid}-${header}`"
                  class="excel-cell"
                  :class="{
                    'error-column-cell': header === errorColumn,
                    'excel-cell-selected':
                      editingCell?.rowIndex === rowIndex && editingCell?.header === header,
                    'excel-cell-context-selected':
                      selectedCellInfo?.rowIndex === rowIndex &&
                      selectedCellInfo?.header === header,
                    'highlighted-column': highlightedColumn === header,
                    'drop-target': dragState.targetIndex === tableHeaders.indexOf(header),
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
                      @click="!previewMode && handleCellClick(rowIndex, header, doc[header])"
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
                  :preview-mode="previewMode"
                  :collection-name="collectionName"
                  :document-id="doc._id.$oid"
                  :row-number="(currentPage - 1) * pageSize + rowIndex + 1"
                  @deleted="fetchDocuments"
                  @delete-start="handleDeleteStart"
                  @delete-end="handleDeleteEnd"
                />
                <StickyTableActions
                  :preview-mode="previewMode"
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
                :class="{ hidden: previewMode }"
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
                :class="{
                  'error-column-cell': header === errorColumn,
                  'highlighted-column': highlightedColumn === header,
                }"
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
              v-if="!isAdding && !previewMode"
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
          v-if="!isAdding && !previewMode"
          class="excel-footer"
        >
          <!-- Only show footer controls when not in preview mode -->
          <span class="excel-page-size-label">Rows per page:</span>
          <Select
            :modelValue="String(pageSize)"
            @update:modelValue="(val) => setPageSize(Number(val))"
            class="excel-page-size-select"
          >
            <SelectTrigger class="w-16"> <SelectValue /> </SelectTrigger>
            <SelectContent>
              <SelectItem value="5">5</SelectItem>
              <SelectItem value="10">10</SelectItem>
              <SelectItem value="20">20</SelectItem>
              <SelectItem value="50">50</SelectItem>
              <SelectItem value="100">100</SelectItem>
            </SelectContent>
          </Select>
          <span class="excel-status-info">
            {{
              props.previewData
                ? `Showing all ${documents.length} entries`
                : `Showing ${paginatedDocuments.length ? (currentPage - 1) * pageSize + 1 : 0} to
           ${(currentPage - 1) * pageSize + paginatedDocuments.length} of
           ${documents.length} entries`
            }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
  .pending-delete-outline {
    background-color: #fee2e2;
    border: 2px solid #ef4444;
    color: #991b1b;
  }
  /* drag state */
  .dragging {
    opacity: 0.6;
    cursor: grabbing;
    background: rgba(33, 150, 243, 0.1);
  }

  .drop-target {
    position: relative;
    border-right: 2px solid #d0d0d0; /* Light gray */
  }
  .drop-target::after {
    content: '';
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 0;
    border-right: 8px solid #d0d0d0; /* Same light gray */
    right: -4px;
  }

  /* highlight for entire column */
  .highlighted-column {
    position: relative;
    outline: none;
    box-shadow:
      1px 0 0 0 #2196f3,
      -1px 0 0 0 #2196f3;
    z-index: 2;
    background-color: rgba(33, 150, 243, 0.05) !important;
  }

  .highlighted-column::after {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    right: -2px;
    width: 2px;
    background-color: #2196f3;
    z-index: 3;
  }

  /* column table head inline edit */
  .header-edit-input {
    @apply px-1 py-0 border border-blue-300 rounded;
  }
  .header-content {
    min-width: 100px;
  }

  .row-resize-handle {
    background-color: transparent;
    transition: background-color 0.2s;
  }

  .row-resize-handle:hover {
    background-color: #3b82f6;
  }

  .excel-data-row:hover .row-resize-handle {
    background-color: #3b82f666;
  }

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
    transition:
      background 0.2s,
      opacity 0.2s;
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
