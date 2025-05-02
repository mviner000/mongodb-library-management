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
  } from 'vue'
  import { useRoute, useRouter } from 'vue-router'
  import { storeToRefs } from 'pinia'

  // Remove useThrottleFn if no longer needed
  import { useDebounceFn } from '@vueuse/core'

  // Store Imports
  import { useDataTableStore } from '@/store/dataTableStore'
  import { useUserStore } from '@/store/useUserStore'

  // UI Component Imports
  import { Cross2Icon, ReloadIcon, PlusCircledIcon } from '@radix-icons/vue'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { ScrollArea } from '@/components/ui/scroll-area'
  import Switch from './ui/switch/Switch.vue'
  import Label from './ui/label/Label.vue'
  import { useToast } from '@/components/ui/toast/use-toast'

  // Custom Component Imports
  import ExcelCellReference from './ExcelCellReference.vue'
  import TableActions from './mongodbtable/TableActions.vue'
  import StickyTableActions from './mongodbtable/StickyTableActions.vue'
  import MongoDBDataTableNavbar from './MongoDBDataTableNavbar.vue'
  import StickyLeftSidebar from './StickyLeftSidebar.vue'
  import CSVCellReference from './CSVCellReference.vue'
  import { getApiBaseUrl, getAuthHeaders } from '@/utils/api'
  import Dialog from './ui/dialog/Dialog.vue'
  import DialogContent from './ui/dialog/DialogContent.vue'
  import DialogTitle from './ui/dialog/DialogTitle.vue'
  import DialogHeader from './ui/dialog/DialogHeader.vue'
  import DialogFooter from './ui/dialog/DialogFooter.vue'
  import ColumnVisibilityControl from './mongodbtable/ColumnVisibilityControl.vue'
  import TableCellContextMenu from './mongodbtable/TableCellContextMenu.vue'
  import AddNewDocumentRow from './mongodbtable/AddNewDocumentRow.vue'

  // ==========================================================================
  // Store Setup
  // ==========================================================================
  const dataTableStore = useDataTableStore()
  const userStore = useUserStore()

  // Destructured State & Getters (Reactive Refs)
  const { user } = storeToRefs(userStore)
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
    columnWidths,
    hiddenColumns,
    previewMode,
    hasMore, // <-- Import hasMore state
    documents, // <-- Make sure documents is imported if used directly
  } = storeToRefs(dataTableStore)

  // Destructured Actions (Functions)
  const {
    fetchCollections,
    setCollection,
    fetchDocuments,
    loadNextPage, // <-- Import loadNextPage action
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
    setPage,
    updateColumnWidth,
    resetColumnWidth,
    saveColumnWidthsToBackend,
    clearError,
    getSchemaInfo,
    isReferenceField,
    getReferencedCollection,
    updateUIMetadata,
    toggleColumnVisibility,
    updateDocumentField,
    pinDocument,
    unpinDocument,
  } = dataTableStore

  // ==========================================================================
  // Props
  // ==========================================================================
  const props = defineProps<{
    selectedCollection?: string
    name?: string
    previewData?: any[]
    dataDisplayMode?: 'valid' | 'invalid'
    // hasMore removed from props since we're using it from the store
  }>()

  // ==========================================================================
  // Local Component State
  // ==========================================================================
  const route = useRoute()
  const router = useRouter()
  const isSplit = inject<Ref<boolean>>('isSplit')!
  const scrollContainer = ref<HTMLElement | null>(null) // Keep if used for scrolling to rows
  const isLoadingMore = ref(false) // Repurposed for the button's loading state
  const isSidebarOpen = ref(false)
  const searchQuery = ref<Record<string, string>>({})
  const selectedCell = ref<{ colIndex: number; rowNumber: number } | null>(null)
  const timeoutId = ref<number | null>(null) // For error message timeout
  const highlightedColumn = ref<string | null>(null)
  const showContextMenu = ref(false)
  const contextMenuPosition = ref({ x: 0, y: 0 })
  const selectedCellInfo = ref<{ rowIndex: number; header: string } | null>(null)
  const highlightedDocumentId = ref<string | null>(null)
  let highlightTimeout: ReturnType<typeof setTimeout> | null = null
  const editingHeader = ref<string | null>(null)
  const editedShortName = ref('')
  const { toast } = useToast()

  // Resizing States
  const resizingState = ref({
    isResizing: false,
    header: '',
    startX: 0,
    startWidth: 0,
    currentWidth: 0,
  })
  const alphaResizingState = ref({
    isResizing: false,
    columnIndex: -1,
    startX: 0,
    startWidth: 0,
    currentWidth: 0,
  })
  const rowResizingState = ref({
    isResizing: false,
    documentId: '',
    startY: 0,
    startHeight: 40, // Default row height
    currentHeight: 40,
  })

  // Drag State
  const dragState = reactive({
    isDragging: false,
    draggedIndex: -1,
    targetIndex: -1,
  })

  // ==========================================================================
  // Computed Properties
  // ==========================================================================
  const paginatedDocuments = computed(() => {
    if (props.previewData) {
      // Preview mode: Show all data without pagination
      return documents.value
    } else {
      // Regular mode: Use store pagination
      const start = (currentPage.value - 1) * pageSize.value
      const end = start + pageSize.value
      return documents.value.slice(start, end)
    }
  })

  const visibleHeaders = computed(() => {
    if (previewMode.value && props.dataDisplayMode === 'invalid') {
      return tableHeaders.value
    }
    return tableHeaders.value.filter((h) => !hiddenColumns.value.includes(h))
  })

  const columnLetters = computed(() => {
    return visibleHeaders.value.map((_, index) => getColumnLabel(index))
  })

  const numberColumnWidth = computed(() => {
    const maxDigits = documents.value.length > 0 ? String(documents.value.length).length : 1
    return `${Math.max(3, maxDigits + 1)}ch`
  })

  const totalTableWidth = computed(() => {
    // Calculate the total width of visible data columns
    const dataColumnsWidth = visibleHeaders.value.reduce(
      (acc, header) => acc + (columnWidths.value[header] || 200), // Default width if not specified
      0
    )

    const selectColWidth = 40 // Width for the selection column
    const rowNumColWidth = 30 // Width for the row number column
    const actionsColWidth = 60 // Width for the actions column

    // Return the total width including only visible columns
    return selectColWidth + rowNumColWidth + dataColumnsWidth + actionsColWidth + 1 // +1 for border
  })

  const isSelectedArchived = computed(() => {
    if (!selectedCellInfo.value) {
      return false
    }

    const rowIndex = selectedCellInfo.value.rowIndex
    const document = paginatedDocuments.value[rowIndex]
    const isArchived = document?.is_archive === true

    return isArchived
  })

  const selectedDocumentIsPinned = computed(() => {
    if (!selectedCellInfo.value) return false
    if (!user.value) return false // Add this check for null user

    const doc = paginatedDocuments.value[selectedCellInfo.value.rowIndex]
    return doc?.pinned_by?.includes(user.value.id)
  })

  const pinnedDocuments = computed(() => {
    if (!user.value) return []
    return documents.value.filter((doc) => doc.pinned_by?.includes(user.value?.id))
  })

  const previewRowHeights = computed(() => {
    if (!previewMode.value || !collectionName.value) return {}
    const heights = sessionStorage.getItem(`previewRowHeights-${collectionName.value}`)
    return heights ? JSON.parse(heights) : {}
  })

  const isImportRoute = computed(() => route.path.includes('/import-csv'))

  // ==========================================================================
  // Watchers
  // ==========================================================================

  // Watch route parameter 'name'
  watch(
    () => route.params.name,
    (newName) => {
      if (previewMode.value) return // Skip in preview
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

  // Watch for error message to auto-clear
  watch(errorMessage, (newVal) => {
    if (newVal) {
      if (timeoutId.value) {
        clearTimeout(timeoutId.value)
      }
      timeoutId.value = setTimeout(() => {
        clearError() // Use store action
        timeoutId.value = null
      }, 2500) as unknown as number
    }
  })

  const emit = defineEmits<{
    (e: 'update:dataDisplayMode', mode: 'valid' | 'invalid'): void
    // 'load-more' emit removed
  }>()

  const tableHeaders = computed(() => {
    if (previewMode.value && props.dataDisplayMode === 'invalid' && props.previewData?.length) {
      const firstItem = props.previewData[0]
      return Object.keys(firstItem).filter((k) => k !== '_id')
    }

    if (!collectionSchema.value.properties) return []
    const schemaProps = collectionSchema.value.properties
    const keys = Object.keys(schemaProps)

    if (collectionSchema.value.ui?.columnOrder) {
      const columnOrder = collectionSchema.value.ui.columnOrder
      // Filter valid headers and add missing ones
      const ordered = columnOrder.filter((key: string) => keys.includes(key))
      const remaining = keys.filter((key: string) => !columnOrder.includes(key))
      return [...ordered, ...remaining]
    }

    return keys.sort((a, b) => {
      const required = collectionSchema.value.required || []
      if (required.includes(a) && !required.includes(b)) return -1
      if (!required.includes(a) && required.includes(b)) return 1
      if (a === '_id') return -1
      if (b === '_id') return 1
      return a.localeCompare(b)
    })
  })

  // Watch isAdding to prefetch reference options
  watch(isAdding, async (newVal) => {
    if (newVal) {
      tableHeaders.value.forEach((field) => {
        const refCollection = getReferencedCollection(field)
        if (refCollection && !referenceOptions.value[refCollection]) {
          fetchReferenceOptions(refCollection)
        }
      })
    }
  })

  // Watch previewMode
  watch(
    previewMode,
    () => {
      // All console.log statements removed
    },
    { immediate: true }
  )

  // Watch route to set preview mode
  watch(
    isImportRoute,
    (newVal) => {
      previewMode.value = newVal
    },
    { immediate: true }
  )

  // ==========================================================================
  // Lifecycle Hooks
  // ==========================================================================
  onMounted(async () => {
    if (previewMode.value) return // Skip fetching in preview mode

    await fetchCollections()
    const routeName = Array.isArray(route.params.name) ? route.params.name[0] : route.params.name
    const initialCollection = routeName || props.selectedCollection

    if (initialCollection && initialCollection !== collectionName.value) {
      await setCollection(initialCollection)
    } else if (collectionName.value && documents.value.length === 0 && !isLoading.value) {
      try {
        await dataTableStore.fetchSchema()
        await dataTableStore.fetchDocuments()
      } catch (error) {
        // Handle error
      }
    }

    // Event listeners
    window.addEventListener('click', closeContextMenu)
    document.addEventListener('click', checkClickOutsideHeaders)
  })

  onBeforeUnmount(() => {
    // Cleanup listeners
    document.removeEventListener('click', checkClickOutsideHeaders)
    window.removeEventListener('click', closeContextMenu)

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
    let label = ''
    let i = index
    do {
      const remainder = i % 26
      label = String.fromCharCode(65 + remainder) + label
      i = Math.floor(i / 26) - 1
    } while (i >= 0)
    return label
  }

  const isFieldRequired = (field: string): boolean => {
    return collectionSchema.value.required?.includes(field) || false
  }

  const formatSchemaValue = (value: any, bsonType?: string | string[]) => {
    if (value === undefined || value === null) return '' // Handle null/undefined early

    const type = bsonType ? (Array.isArray(bsonType) ? bsonType[0] : bsonType) : typeof value

    // Handle empty strings for non-string types
    if (typeof value === 'string' && value.trim() === '') {
      return ''
    }

    if (type === 'date') {
      try {
        return value instanceof Date ? value.toLocaleString() : new Date(value).toLocaleString()
      } catch {
        return '' // Return empty for invalid dates
      }
    }

    if (typeof value === 'object') {
      return JSON.stringify(value, null, 2)
    }

    return String(value)
  }

  const filteredOptions = (field: string) => {
    const refCollection = getReferencedCollection(field)
    if (!refCollection) return []
    const options = referenceOptions.value[refCollection] || []
    const query = (searchQuery.value[field] || '').toLowerCase()
    return options.filter((opt) => opt.label.toLowerCase().includes(query))
  }

  const shortName = (header: string) => {
    return collectionSchema.value.ui?.short_names?.[header] || header
  }

  function getActualIndex(visibleIndex: number) {
    let actualIndex = 0
    let visibleCount = 0

    while (visibleCount <= visibleIndex && actualIndex < tableHeaders.value.length) {
      if (!hiddenColumns.value.includes(tableHeaders.value[actualIndex])) {
        visibleCount++
      }
      actualIndex++
    }

    return actualIndex - 1
  }

  // ==========================================================================
  // Debounced Functions
  // ==========================================================================
  const debouncedSaveWidths = useDebounceFn(async () => {
    await saveColumnWidthsToBackend() // Call store action
  }, 750) // Keep debounce consistent

  const debouncedRowHeightSave = useDebounceFn(async (documentId: string, height: number) => {
    try {
      await updateDocumentField(documentId, 'row_height', height)
    } catch (error) {
      // Error handling without console.error
    }
  }, 500) // 500ms delay

  // ==========================================================================
  // Methods
  // ==========================================================================

  // --- Error Handling ---
  const closeErrorManual = () => {
    if (timeoutId.value) {
      clearTimeout(timeoutId.value)
      timeoutId.value = null
    }
    clearError() // Use store action
  } // Matches original @click="closeError"

  // --- Column Resizing ---
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
  }

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
  }

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
  }

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

  // --- Row Resizing ---
  const startRowResize = (documentId: string, event: MouseEvent) => {
    const doc = documents.value.find((d) => d._id.$oid === documentId)
    if (!doc) return

    const currentHeight = doc.row_height || 40
    rowResizingState.value = {
      isResizing: true,
      documentId,
      startY: event.clientY,
      startHeight: currentHeight,
      currentHeight,
    }

    document.addEventListener('mousemove', handleRowMouseMove)
    document.addEventListener('mouseup', stopRowResize)
  }

  const handleRowMouseMove = (event: MouseEvent) => {
    if (!rowResizingState.value.isResizing) return

    const delta = event.clientY - rowResizingState.value.startY
    const newHeight = Math.max(40, rowResizingState.value.startHeight + delta)

    rowResizingState.value.currentHeight = newHeight

    // Update local document for visual feedback
    const docIndex = documents.value.findIndex(
      (d) => d._id.$oid === rowResizingState.value.documentId
    )
    if (docIndex !== -1) {
      const updatedDoc = {
        ...documents.value[docIndex],
        row_height: newHeight,
      }
      documents.value.splice(docIndex, 1, updatedDoc)
    }
  }

  const stopRowResize = async () => {
    if (!rowResizingState.value.isResizing) return
    const { documentId, currentHeight } = rowResizingState.value

    rowResizingState.value.isResizing = false
    document.removeEventListener('mousemove', handleRowMouseMove)
    document.removeEventListener('mouseup', stopRowResize)

    if (previewMode.value) {
      const newHeights = {
        ...previewRowHeights.value,
        [documentId]: currentHeight,
      }
      sessionStorage.setItem(
        `previewRowHeights-${collectionName.value}`,
        JSON.stringify(newHeights)
      )
    } else {
      debouncedRowHeightSave(documentId, currentHeight)
    }
  }

  // --- Cell Click/Edit Handling ---
  const handleCellClick = (rowIndex: number, header: string, value: any) => {
    // Value param from original template
    if (isSaving.value || ['_id', 'created_at', 'updated_at'].includes(header)) return

    // Use the original value passed from the template click event
    startEditingCell(rowIndex, header, value) // Use store action

    // Update selectedCell for visual feedback (Excel-like)
    const actualRowNumber = (currentPage.value - 1) * pageSize.value + rowIndex + 1
    const colIndex = tableHeaders.value.indexOf(header)
    selectedCell.value = { colIndex, rowNumber: actualRowNumber }

    // Focus logic
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
  } // Combines original template call with store logic

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
  } // Connects to @blur event

  // --- Header Editing ---
  const startEditingHeader = (header: string) => {
    editingHeader.value = header
    editedShortName.value = shortName(header)
  }

  const isEditingHeader = (header: string) => {
    return editingHeader.value === header
  }

  const saveShortName = async (header: string) => {
    if (!editedShortName.value.trim()) return

    const newShortNames = {
      ...collectionSchema.value.ui?.short_names,
      [header]: editedShortName.value,
    }

    try {
      await updateUIMetadata({ short_names: newShortNames })
      editingHeader.value = null
    } catch (error) {
      // Handle error
    }
  }

  const cancelEditTextHead = () => {
    editingHeader.value = null
  }

  // --- Column Highlighting ---
  const handleColumnHighlight = (index: number) => {
    // Count hidden columns before the visible index to get the actual index in tableHeaders
    let hiddenColumnsCount = 0
    for (let i = 0; i <= index; i++) {
      const visibleIndex = i + hiddenColumnsCount
      if (
        visibleIndex < tableHeaders.value.length &&
        hiddenColumns.value.includes(tableHeaders.value[visibleIndex])
      ) {
        hiddenColumnsCount++
        i-- // Stay at the same visible index since we found a hidden column
      }
    }

    // Adjust index to account for hidden columns
    const adjustedIndex = index + hiddenColumnsCount
    const header = tableHeaders.value[adjustedIndex]

    // Clear both selected cell and editing state
    selectedCell.value = null
    cancelEdit() // This clears editingCell in the store

    if (selectedRows.value.size > 0) {
      const confirmed = confirm('You have selected rows. Unselect them to highlight the column?')

      if (confirmed) {
        resetSelection()
        highlightedColumn.value = header
      }
    } else {
      if (header === highlightedColumn.value) {
        highlightedColumn.value = null
      } else {
        highlightedColumn.value = header
      }
    }
  }

  const checkClickOutsideHeaders = (event: MouseEvent) => {
    const target = event.target as HTMLElement
    const isHeaderClick = target.closest('.excel-column-letter, .excel-column-header')
    if (!isHeaderClick) {
      highlightedColumn.value = null
    }
  }

  // --- Drag and Drop Columns ---
  function onDragStart(event: DragEvent, visibleIndex: number) {
    const actualIndex = getActualIndex(visibleIndex)
    dragState.isDragging = true
    dragState.draggedIndex = actualIndex
    event.dataTransfer?.setData('text/plain', '')
    highlightedColumn.value = tableHeaders.value[actualIndex]
  }

  function onDragOver(event: DragEvent, visibleIndex: number) {
    if (!dragState.isDragging) return
    event.preventDefault()

    const actualIndex = getActualIndex(visibleIndex)
    if (dragState.targetIndex !== actualIndex) {
      dragState.targetIndex = actualIndex
    }
  }

  function onDrop(event: DragEvent, visibleIndex: number) {
    event.preventDefault()
    if (!dragState.isDragging) return

    const targetActualIndex = getActualIndex(visibleIndex)
    const draggedIndex = dragState.draggedIndex

    // Existing column order logic
    const currentUI = collectionSchema.value.ui || {}
    let columnOrder = currentUI.columnOrder ? [...currentUI.columnOrder] : [...tableHeaders.value]
    const allHeaders = Object.keys(collectionSchema.value.properties || {})

    // Clean and reorder
    columnOrder = columnOrder.filter((h) => allHeaders.includes(h))
    const missing = allHeaders.filter((h) => !columnOrder.includes(h))
    columnOrder.push(...missing)

    // Perform the move using actual indices
    const [moved] = columnOrder.splice(draggedIndex, 1)
    columnOrder.splice(targetActualIndex, 0, moved)

    // Update UI metadata
    updateUIMetadata({ columnOrder })

    // Reset state
    dragState.isDragging = false
    dragState.draggedIndex = -1
    dragState.targetIndex = -1
    highlightedColumn.value = null
  }

  function onDragEnd() {
    dragState.isDragging = false
    dragState.draggedIndex = -1
    dragState.targetIndex = -1
  }

  // --- Context Menu ---
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
    if (!selectedCellInfo.value) {
      return
    }
    if (isSelectedArchived.value) {
      return
    }
    // Add check for null user
    if (!user.value) {
      return
    }
    const rowIndex = selectedCellInfo.value.rowIndex
    const doc = paginatedDocuments.value[rowIndex]
    if (!doc) {
      return
    }
    const isPinned = doc.pinned_by?.includes(user.value.id)
    try {
      if (isPinned) {
        await unpinDocument(doc._id.$oid)
      } else {
        await pinDocument(doc._id.$oid)
      }

      // Add this critical line to refresh documents after pin/unpin operation
      await fetchDocuments()
    } catch (error) {
      // Error handling without console.error
    }
    closeContextMenu()
  }

  const bookmarkCell = () => {
    if (!selectedCellInfo.value) return
    closeContextMenu()
  }

  const togglePinStatus = async (docId: string, currentPinStatus: boolean): Promise<void> => {
    try {
      if (currentPinStatus) {
        await unpinDocument(docId)
      } else {
        await pinDocument(docId)
      }
    } catch (error) {
      // Error handling without console.error
    }
  }

  // --- Document Navigation/Highlighting ---
  const handleDocumentNavigation = (docId: string) => {
    // Clear existing timeout if any
    if (highlightTimeout) clearTimeout(highlightTimeout)

    // Find document index in full dataset
    const index = documents.value.findIndex((doc) => doc._id.$oid === docId)
    if (index === -1) return

    // Calculate and set correct page
    const page = Math.ceil((index + 1) / pageSize.value)
    setPage(page)

    // Set highlight and auto-clear after 2s
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

  // --- Template specific handlers ---
  const handleViewChange = (view: string) => {
    changeView(view) // Use store action
  }

  const handleDeleteStart = (id: string) => {
    dataTableStore.pendingDeleteId = id
  }

  const handleDeleteEnd = () => {
    dataTableStore.pendingDeleteId = null
  }

  // ==========================================================================
  // Expose (if needed by parent components)
  // ==========================================================================
  defineExpose({
    fetchDocuments: dataTableStore.fetchDocuments,
    fetchCollections: dataTableStore.fetchCollections,
    setCollection: dataTableStore.setCollection,
  })

  const localAllSelected = computed({
    get: () => {
      if (documents.value.length === 0) return false
      return documents.value.every((doc) => selectedRows.value.has(doc._id.$oid))
    },
    set: (val: boolean) => {
      if (val) {
        const ids = documents.value.map((doc) => doc._id.$oid)
        selectedRows.value = new Set(ids)
      } else {
        selectedRows.value.clear()
      }
    },
  })

  // state for download options
  const showDownloadDialog = ref(false)
  const downloadHeaderChoice = ref<'short' | 'original'>('short')
  const includeId = ref(false) // New include _id option

  const downloadCSV = async () => {
    console.debug('downloadCSV: Handler called')
    showDownloadDialog.value = false
    try {
      // Add include_id parameter to URL
      const url = `${getApiBaseUrl()}/collections/${collectionName.value}/download-csv?headers=${downloadHeaderChoice.value}&include_id=${includeId.value}`
      console.debug('downloadCSV: Fetching from URL:', url)

      const response = await fetch(url, {
        headers: getAuthHeaders(),
      })

      if (!response.ok) throw new Error('Download failed')

      const blob = await response.blob()
      const downloadUrl = window.URL.createObjectURL(blob)

      // Generate dynamic filename
      const now = new Date()
      const timestamp = [
        now.getFullYear(),
        (now.getMonth() + 1).toString().padStart(2, '0'),
        now.getDate().toString().padStart(2, '0'),
        '_',
        now.getHours().toString().padStart(2, '0'),
        now.getMinutes().toString().padStart(2, '0'),
        now.getSeconds().toString().padStart(2, '0'),
      ].join('')

      const headerType =
        downloadHeaderChoice.value === 'short' ? 'short_name_is_used' : 'orig_name_is_used'
      const idSuffix = includeId.value ? '_with_id' : ''

      const filename = `${collectionName.value}_${timestamp}_${headerType}${idSuffix}.csv`

      const a = document.createElement('a')
      a.href = downloadUrl
      a.download = filename
      document.body.appendChild(a)
      a.click()
      window.URL.revokeObjectURL(downloadUrl)
      document.body.removeChild(a)

      // Show success toast
      toast({
        title: 'Download Successful',
        description: 'CSV file downloaded successfully',
        variant: 'default',
      })
    } catch (error) {
      // Error handling and toast
      const message = error instanceof Error ? error.message : 'Unknown error'
      errorMessage.value = `Download failed: ${message}`
      toast({
        title: 'Download Error',
        description: errorMessage.value,
        variant: 'destructive',
      })
    }
  }

  const isOnImportPage = route.path.endsWith('/import-csv')
  const handleShowMoreClick = async () => {
    console.log('=== SHOW MORE DEBUG START ===')
    console.log('handleShowMoreClick: Function called')
    console.log('Current state before loading more:', {
      currentPage: currentPage.value,
      pageSize: pageSize.value,
      documentsCount: documents.value.length,
      hasMore: hasMore.value,
      isLoadingMore: isLoadingMore.value,
    })

    isLoadingMore.value = true // Show loading state
    console.log('Loading state set to true')

    try {
      console.log('Calling loadNextPage action...')
      await loadNextPage() // Call the store action
      console.log('loadNextPage completed')
      console.log('State after loading more:', {
        currentPage: currentPage.value,
        pageSize: pageSize.value,
        documentsCount: documents.value.length,
        hasMore: hasMore.value,
        isLoadingMore: isLoadingMore.value,
      })
    } catch (error) {
      console.error('Error in handleShowMoreClick:', error)
      // Type-safe error handling
      if (error instanceof Error) {
        console.log('Error details:', {
          message: error.message,
          stack: error.stack,
        })
      } else {
        console.log('Unknown error type:', error)
      }
    } finally {
      isLoadingMore.value = false // Hide loading state
      console.log('Loading state set back to false')
      console.log('=== SHOW MORE DEBUG END ===')
    }
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
          :previewMode="!previewMode"
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
        <!-- Column Controls -->
        <div class="z-40 mt-10 p-2 bg-[#217346]/90 flex justify-between">
          <!-- Left side content -->
          <div class="flex gap-2">
            <div class="flex items-center space-x-2 hidden">
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

            <div class="flex gap-2 ml-4">
              <router-link
                v-if="isOnImportPage"
                :to="`/collection/${collectionName}`"
              >
                <Button
                  variant="outline"
                  size="sm"
                >
                  Back
                </Button>
              </router-link>

              <router-link
                v-else
                :to="`/collection/${collectionName}/import-csv`"
              >
                <Button
                  variant="outline"
                  size="sm"
                >
                  Import CSV
                </Button>
              </router-link>
            </div>

            <div class="flex gap-2">
              <Select
                v-if="isOnImportPage"
                :modelValue="props.dataDisplayMode"
                @update:modelValue="
                  (value) => emit('update:dataDisplayMode', value as 'valid' | 'invalid')
                "
              >
                <SelectTrigger class="w-[120px]">
                  <SelectValue placeholder="Data Type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="valid">Valid</SelectItem>
                  <SelectItem value="invalid">Invalid</SelectItem>
                </SelectContent>
              </Select>

              <Button @click="showDownloadDialog = true"> Download CSV Data </Button>
            </div>

            <!-- Download Dialog -->
            <Dialog v-model:open="showDownloadDialog">
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Download Options</DialogTitle>
                </DialogHeader>
                <div class="space-y-4">
                  <div>
                    <Label>Header Row Format:</Label>
                    <Select v-model="downloadHeaderChoice">
                      <SelectTrigger>
                        <SelectValue placeholder="Select header format" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="short">Short Names</SelectItem>
                        <SelectItem value="original">Original Names</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div class="flex items-center space-x-2">
                    <Switch
                      id="include-id"
                      :modelValue="includeId"
                      @update:modelValue="(value) => (includeId = value)"
                    />
                    <Label
                      for="include-id"
                      class="cursor-pointer"
                    >
                      Include ID
                    </Label>
                  </div>
                </div>
                <DialogFooter>
                  <Button @click="downloadCSV">Download</Button>
                  <Button
                    variant="outline"
                    @click="showDownloadDialog = false"
                    >Cancel</Button
                  >
                </DialogFooter>
              </DialogContent>
            </Dialog>
          </div>

          <!-- Right side content -->
          <div class="ml-auto mt-1">
            <ColumnVisibilityControl
              :table-headers="tableHeaders"
              :hidden-columns="hiddenColumns"
              :collection-schema="collectionSchema"
              @toggle-visibility="toggleColumnVisibility"
            />
          </div>
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
                <!-- Select All/Unselect All -->
                <input
                  type="checkbox"
                  :checked="localAllSelected"
                  @change="localAllSelected = !localAllSelected"
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
              <TableCellContextMenu
                :is-visible="showContextMenu"
                :position="contextMenuPosition"
                :is-pinned="selectedDocumentIsPinned"
                :is-archived="isSelectedArchived"
                :is-preview="previewMode"
                @close="closeContextMenu"
                @pin="pinCell"
                @bookmark="bookmarkCell"
              />

              <!-- End of Context Menu -->
            </template>

            <!-- start of add new document component -->
            <AddNewDocumentRow
              v-if="isAdding"
              :table-headers="visibleHeaders"
              :new-document="newDocument"
              @update:newDocument="(payload) => (newDocument = payload)"
              :is-saving="isSaving"
              :error-column="errorColumn"
              :adding-row-error="addingRowError"
              :highlighted-column="highlightedColumn"
              :total-documents="documents.length"
              :get-schema-info="getSchemaInfo"
              :is-reference-field="isReferenceField"
              :get-referenced-collection="getReferencedCollection"
              :reference-options="referenceOptions"
              :loading-references="loadingReferences"
              :search-query="searchQuery"
              @save="saveNewDocument"
              @cancel="cancelAdding"
            />

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
            <!-- end of add new document component -->
          </TableBody>
        </table>
        fsdfdfd

        <Button
          @click="handleShowMoreClick"
          :disabled="isLoading || isLoadingMore"
          variant="outline"
        >
          <ReloadIcon
            v-if="isLoading || isLoadingMore"
            class="mr-2 h-4 w-4 animate-spin"
          />
          {{ isLoading || isLoadingMore ? 'Loading...' : 'Show More' }}
        </Button>

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

        <!-- Loading indicator -->
        <div
          v-if="isLoadingMore"
          class="loading-indicator"
        >
          <ReloadIcon class="h-6 w-6 animate-spin text-gray-500" />
        </div>
      </div>

      <!-- loading indicator -->
      <div
        v-if="isLoadingMore"
        class="loading-indicator"
      >
        <ReloadIcon class="h-6 w-6 animate-spin text-gray-500" />
      </div>
    </div>
  </div>
</template>

<style scoped>
  .loading-indicator {
    display: flex;
    justify-content: center;
    padding: 1rem;
  }
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

  /* ... (styles for .excel-select, .excel-input, .excel-checkbox etc. are general, */
  /* but their application within .excel-new-row is specific) ... */
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
    height: 16px; /* [cite: 141] */
    width: 12px; /* Added width */
    background-color: #f0fdf4; /* [cite: 141] */
  }

  .table-scroll-container::-webkit-scrollbar-track {
    background: #f0fdf4; /* [cite: 142] */
    border-radius: 10px; /* [cite: 142] */
  }

  .table-scroll-container::-webkit-scrollbar-thumb {
    background: #16a34a; /* [cite: 142] */
    border-radius: 10px; /* [cite: 142] */
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
