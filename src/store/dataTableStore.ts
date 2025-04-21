// src/store/dataTableStore.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useToast } from '@/components/ui/toast/use-toast'
import { getApiBaseUrl } from '@/utils/api'
import { documentService } from '@/services/documentService'
import { useDebounceFn } from '@vueuse/core'
const API_BASE = getApiBaseUrl()

// Define the structure of a document (adjust based on your actual data)
interface Document {
  _id: { $oid: string }
  [key: string]: any // Allow other properties
}

// Define the structure for reference options
interface ReferenceOption {
  id: string
  label: string
}

export const useDataTableStore = defineStore('dataTable', () => {
  const previewMode = ref(sessionStorage.getItem('previewMode') === 'true')
  const { toast } = useToast()

  // --- State ---
  const collectionName = ref<string>('users') // Default collection
  const documents = ref<Document[]>([])
  const collectionSchema = ref<any>({}) // Consider defining a stricter type
  const isLoading = ref<boolean>(false)
  const errorMessage = ref<string>('')
  const pageSize = ref<number>(20)
  const currentPage = ref<number>(1)
  const hasMore = ref<boolean>(true)
  const filterQuery = ref<string>('{}') // Keep filter query local or move if needed globally
  const newDocument = ref<Record<string, any>>({})
  const isAdding = ref<boolean>(false)
  const editingCell = ref<{ rowIndex: number; header: string } | null>(null)
  const editValue = ref<any>('') // Can be string, boolean, etc.
  const isSaving = ref<boolean>(false)
  const selectedRows = ref<Set<string>>(new Set())
  const currentView = ref<string>('empty-or-recovered') // e.g., "all", "archives"
  const pendingDeleteId = ref<string | null>(null)
  const referenceOptions = ref<Record<string, ReferenceOption[]>>({})
  const loadingReferences = ref<Record<string, boolean>>({})
  const collectionsList = ref<string[]>([])
  const errorColumn = ref<string | null>(null) // For highlighting duplicate errors
  const addingRowError = ref<boolean>(false)

  // --- Getters (Computed) ---
  const totalDocuments = ref<number>(0)
  const totalPages = computed(() => Math.ceil(totalDocuments.value / pageSize.value))

  const paginatedDocuments = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value
    const end = start + pageSize.value
    return documents.value.slice(start, end)
  })

  const tableHeaders = computed(() => {
    if (!collectionSchema.value.properties) return []
    const props = collectionSchema.value.properties
    const keys = Object.keys(props)

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

  const columnWidths = computed(() => {
    return collectionSchema.value?.ui?.columnWidths || {}
  })

  const allSelected = computed({
    get: () => totalDocuments.value > 0 && selectedRows.value.size === totalDocuments.value,
    set: (val: boolean) => {
      selectedRows.value = val ? new Set(documents.value.map((doc) => doc._id.$oid)) : new Set()
    },
  })

  // --- Actions ---

  // Helper to get schema info
  const getSchemaInfo = (field: string) => {
    return collectionSchema.value.properties?.[field] || {}
  }

  // Helper to check if field is reference
  const isReferenceField = (field: string): boolean => {
    return getSchemaInfo(field).description?.includes('REF:') || false
  }

  // Helper to get referenced collection name
  const getReferencedCollection = (field: string): string | null => {
    const desc = getSchemaInfo(field).description || ''
    const match = desc.match(/REF:(\w+)/)
    return match ? match[1] : null
  }

  // Fetch list of collections
  async function fetchCollections() {
    try {
      const response = await fetch(`${API_BASE}/collections`)
      if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`)
      const { success, data, error } = await response.json()
      if (success) {
        collectionsList.value = data
        // Optionally set collectionName if current is invalid or empty
        if (data.length > 0 && (!collectionName.value || !data.includes(collectionName.value))) {
          // await setCollection(data[0]); // Uncomment if you want to auto-select
        }
      } else {
        throw new Error(error || 'Failed to fetch collections')
      }
    } catch (err: any) {
      errorMessage.value = `Error fetching collections: ${err.message}`
    }
  }

  // Set the current collection and fetch its data
  async function setCollection(name: string) {
    if (collectionName.value === name && documents.value.length > 0) return // Avoid refetch if already set

    collectionName.value = name
    // Reset state for the new collection
    documents.value = []
    collectionSchema.value = {}
    currentPage.value = 1
    selectedRows.value = new Set()
    editingCell.value = null
    isAdding.value = false
    errorMessage.value = ''
    referenceOptions.value = {} // Clear old references

    await fetchSchema() // Fetch schema first

    // Initialize hidden columns from schema
    hiddenColumns.value = collectionSchema.value.ui?.hiddenColumns || []

    await fetchDocuments() // Then fetch documents
    await preloadReferenceOptions() // Preload references needed by the schema
  }

  const pinDocument = async (documentId: string) => {
    if (!collectionName.value) {
      return
    }

    try {
      const response = await documentService.pinDocument(collectionName.value, documentId)

      if (response.success) {
        // Find and replace the document with the updated version from the response
        const updatedDoc = response.data
        const index = documents.value.findIndex((doc) => doc._id.$oid === documentId)

        if (index !== -1) {
          documents.value[index] = updatedDoc
        } else {
          // If not found, add to documents array (might be a new pinned document)
          documents.value.push(updatedDoc)
        }

        // Force array update for Vue reactivity
        documents.value = [...documents.value]

        toast({ title: 'Pinned', description: 'Document pinned successfully' })
      } else {
        errorMessage.value = `Error pinning document: ${response.error}`
        toast({ title: 'Pin Error', description: errorMessage.value, variant: 'destructive' })
      }
    } catch (err: any) {
      errorMessage.value = `Error pinning document: ${err.message}`
      toast({ title: 'Pin Error', description: errorMessage.value, variant: 'destructive' })
    }
  }

  const unpinDocument = async (documentId: string) => {
    if (!collectionName.value) {
      return
    }

    try {
      const response = await documentService.unpinDocument(collectionName.value, documentId)

      if (response.success) {
        // Find and replace the document with the updated version from the response
        const updatedDoc = response.data
        const index = documents.value.findIndex((doc) => doc._id.$oid === documentId)

        if (index !== -1) {
          documents.value[index] = updatedDoc
        } else {
          // If not found, add to documents array (might be a new pinned document)
        }

        // Force array update for Vue reactivity
        documents.value = [...documents.value]

        toast({ title: 'Unpinned', description: 'Document unpinned successfully' })
      } else {
        errorMessage.value = `Error unpinning document: ${response.error}`
        toast({ title: 'Unpin Error', description: errorMessage.value, variant: 'destructive' })
      }
    } catch (err: any) {
      errorMessage.value = `Error unpinning document: ${err.message}`
      toast({ title: 'Unpin Error', description: errorMessage.value, variant: 'destructive' })
    }
  }

  // Fetch schema for the current collection
  async function fetchSchema() {
    if (previewMode.value) return // Skip in preview
    if (!collectionName.value) return

    isLoading.value = true // Indicate loading schema
    errorMessage.value = ''

    try {
      // Prefer API fetch if available, fallback to invoke if needed
      const response = await fetch(`${API_BASE}/collections/${collectionName.value}/schema`)
      if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`)
      const { success, data, error } = await response.json()

      if (success) {
        collectionSchema.value = data

        // Apply preview state if in preview mode
        if (previewMode.value) {
          const savedUI = sessionStorage.getItem(`previewState-${collectionName.value}`)
          if (savedUI) {
            collectionSchema.value.ui = {
              ...collectionSchema.value.ui,
              ...JSON.parse(savedUI),
            }
          }
        }

        initializeNewDocument() // Initialize based on new schema
      } else {
        throw new Error(error || 'Schema fetch failed')
      }
    } catch (err: any) {
      errorMessage.value = `Schema error: ${err.message}`
      collectionSchema.value = {} // Reset schema on error
    } finally {
      isLoading.value = false // Schema loading finished
    }
  }

  // Fetch documents for the current collection and view
  async function fetchDocuments() {
    if (previewMode.value) return
    if (!collectionName.value) return

    isLoading.value = true
    errorMessage.value = ''
    pendingDeleteId.value = null

    try {
      let filter = {}
      try {
        filter = JSON.parse(filterQuery.value)
      } catch (e: any) {
        throw new Error(`Invalid filter JSON: ${e.message}`)
      }

      let endpoint
      switch (currentView.value) {
        case 'archives':
          endpoint = `${API_BASE}/collections/${collectionName.value}/archives`
          break
        case 'recoveries':
          endpoint = `${API_BASE}/collections/${collectionName.value}/recoveries`
          break
        case 'empty-or-recovered':
          endpoint = `${API_BASE}/collections/${collectionName.value}/empty-or-recovered`
          break
        case 'all':
        default:
          endpoint = `${API_BASE}/collections/${collectionName.value}/documents`
          break
      }

      const params = new URLSearchParams()
      params.append('filter', JSON.stringify(filter))
      params.append('page', currentPage.value.toString())
      params.append('limit', pageSize.value.toString())

      const url = `${endpoint}?${params.toString()}`
      const response = await fetch(url)

      if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`)

      const { success, data: responseData, error } = await response.json()

      if (success) {
        // Append new documents or replace if first page
        if (currentPage.value === 1) {
          documents.value = responseData.items
        } else {
          documents.value.push(...responseData.items)
        }

        totalDocuments.value = responseData.total
        hasMore.value = currentPage.value * pageSize.value < responseData.total

        // Reset page if no results
        if (currentPage.value > 1 && responseData.items.length === 0) {
          currentPage.value--
        }
      } else {
        throw new Error(error || 'Failed to fetch documents')
      }
    } catch (err: any) {
      errorMessage.value = `Error fetching documents: ${err.message}`
      if (currentPage.value > 1) currentPage.value--
    } finally {
      isLoading.value = false
    }
  }

  // --- New function to load next page ---
  async function loadNextPage() {
    if (!hasMore.value || isLoading.value) return
    currentPage.value++
    await fetchDocuments()
  }

  // --- Reset pagination when view changes ---
  function changeView(view: string) {
    if (currentView.value !== view) {
      currentView.value = view
      currentPage.value = 1
      documents.value = []
      hasMore.value = true
      fetchDocuments()
    }
  }

  // Preload reference options needed for the current schema
  async function preloadReferenceOptions() {
    if (!collectionSchema.value.properties) return

    const fields = Object.keys(collectionSchema.value.properties)
    for (const field of fields) {
      const refCollection = getReferencedCollection(field)
      if (refCollection && !referenceOptions.value[refCollection]) {
        await fetchReferenceOptions(refCollection)
      }
    }
  }

  // Fetch options for a referenced collection
  async function fetchReferenceOptions(refCollectionName: string, force = false) {
    if (!force && referenceOptions.value[refCollectionName]?.length > 0) {
      return // Already fetched or has data
    }
    if (loadingReferences.value[refCollectionName]) return // Already loading

    loadingReferences.value[refCollectionName] = true

    try {
      // 1. Fetch Schema to determine the best label field
      const schemaResponse = await fetch(`${API_BASE}/collections/${refCollectionName}/schema`)
      if (!schemaResponse.ok) throw new Error(`Schema fetch failed for ${refCollectionName}`)
      const {
        success: schemaSuccess,
        data: schemaData,
        error: schemaError,
      } = await schemaResponse.json()
      if (!schemaSuccess)
        throw new Error(schemaError || `Failed to fetch schema for ${refCollectionName}`)

      let labelField = '_id' // Default label
      const properties = schemaData.properties || {}
      const uniqueStringFields = Object.keys(properties).filter(
        (field) => properties[field].bsonType === 'string' && properties[field].unique === true
      )
      if (uniqueStringFields.length > 0) {
        labelField = uniqueStringFields[0] // Prefer unique string fields
      } else {
        const commonLabels = ['label', 'name', 'title', 'username']
        labelField = commonLabels.find((f) => properties[f]?.bsonType === 'string') || '_id'
      }

      // 2. Fetch Documents to populate options
      const docsResponse = await fetch(
        `${API_BASE}/collections/${refCollectionName}/documents?limit=1000`
      ) // Fetch all or limit as needed
      if (!docsResponse.ok) throw new Error(`Document fetch failed for ${refCollectionName}`)
      const { success: docsSuccess, data: docsData, error: docsError } = await docsResponse.json()

      if (docsSuccess) {
        // Check if paginated structure exists
        const items = docsData.items || docsData

        referenceOptions.value[refCollectionName] = items.map((doc: any) => ({
          id: doc._id.$oid, // Assuming MongoDB ObjectId structure
          label: doc[labelField] || doc._id.$oid, // Use determined label or fallback to ID
        }))

        if (referenceOptions.value[refCollectionName].length === 0) {
          toast({
            title: 'No Reference Options',
            description: `No documents found in collection '${refCollectionName}'.`,
            variant: 'default',
          })
        }
      } else {
        throw new Error(docsError || `Failed to fetch documents for ${refCollectionName}`)
      }
    } catch (err: any) {
      toast({
        title: 'Reference Error',
        description: `Failed to load options for ${refCollectionName}: ${err.message}`,
        variant: 'destructive',
      })
      referenceOptions.value[refCollectionName] = [] // Ensure it's an empty array on error
    } finally {
      loadingReferences.value[refCollectionName] = false
    }
  }

  // Get label for a reference ID
  function getReferenceLabel(field: string, id: string): string {
    const refCollection = getReferencedCollection(field)
    if (!refCollection) return id // Not a reference field
    const options = referenceOptions.value[refCollection] || []
    const option = options.find((opt) => opt.id === id)
    return option ? option.label : id // Return label or ID if not found
  }

  // Initialize the new document structure based on schema
  function initializeNewDocument() {
    const doc: Record<string, any> = {}
    const required = collectionSchema.value.required || []
    const properties = collectionSchema.value.properties || {}

    // Initialize required fields first (excluding specific auto-fields)
    required.forEach((field: string) => {
      if (['_id', 'created_at', 'updated_at'].includes(field)) return
      const prop = properties[field]
      if (prop) {
        doc[field] = getDefaultValue(prop.bsonType)
      }
    })

    // Consider adding default values for non-required fields if desired
    // Object.keys(properties).forEach(field => {
    //   if (!doc.hasOwnProperty(field) && !['_id', 'created_at', 'updated_at'].includes(field)) {
    //      doc[field] = getDefaultValue(properties[field].bsonType);
    //   }
    // });

    newDocument.value = doc
  }

  // Get default value based on BSON type
  function getDefaultValue(bsonType: string | string[]) {
    const type = Array.isArray(bsonType) ? bsonType[0] : bsonType // Handle potential array of types
    switch (type) {
      case 'string':
        return ''
      case 'int':
      case 'long':
      case 'double':
      case 'decimal':
        return 0
      case 'bool':
        return false
      case 'date':
        return new Date().toISOString() // Store dates consistently, e.g., ISO string
      case 'objectId':
        return null // Or handle differently if needed
      case 'object':
        return {}
      case 'array':
        return []
      default:
        return null
    }
  }

  // Start adding a new document
  function startAdding() {
    initializeNewDocument() // Ensure clean slate based on current schema
    isAdding.value = true
    addingRowError.value = false // Reset error state
    errorColumn.value = null
    // Pre-fetch references needed for the add form
    preloadReferenceOptions()
  }

  // Cancel adding
  function cancelAdding() {
    isAdding.value = false
    newDocument.value = {} // Clear the form
    addingRowError.value = false
    errorColumn.value = null
  }

  // Save the new document
  async function saveNewDocument() {
    if (!collectionName.value) return
    isSaving.value = true // Use isSaving for feedback
    errorMessage.value = ''
    errorColumn.value = null
    addingRowError.value = false

    try {
      const response = await fetch(`${API_BASE}/collections/${collectionName.value}/documents`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newDocument.value),
      })

      // Check if response is OK, even if not strictly JSON (e.g., 201 Created with no body)
      if (!response.ok) {
        // Try to parse error JSON, otherwise use status text
        let errorData
        try {
          errorData = await response.json()
        } catch (jsonError) {
          // Ignore json parse error if body is not json
        }
        const errorDetail = errorData?.error || `HTTP ${response.status}: ${response.statusText}`
        throw new Error(errorDetail)
      }

      // Handle potential non-JSON success response (like 201)
      let result
      const contentType = response.headers.get('content-type')
      if (contentType && contentType.includes('application/json')) {
        result = await response.json()
      } else {
        // Assume success if response.ok was true and no JSON body
        result = { success: true }
      }

      // Check application-level success (if applicable)
      if (result.success) {
        toast({ title: 'Success', description: 'Document created successfully.' })
        isAdding.value = false
        await fetchDocuments() // Refresh list
      } else {
        // This else block might be redundant if !response.ok is handled above,
        // but keep it if your API returns 200 OK with a success: false payload
        throw new Error(result.error || 'Failed to create document (API indicated failure)')
      }
    } catch (err: any) {
      errorMessage.value = err.message || 'Failed to create document'
      addingRowError.value = true // Indicate error on the add row

      // Handle specific errors like duplicates
      if (err.message && err.message.includes('E11000')) {
        // Extract field name from duplicate key error message
        const fieldMatch = err.message.match(/index: (\w+)_/) // Basic pattern, adjust if needed
        const valueMatch = err.message.match(/dup key: { (\w+): "(.+?)" }/) // More specific pattern
        const fieldName = valueMatch ? valueMatch[1] : fieldMatch ? fieldMatch[1] : null

        if (fieldName) {
          errorMessage.value = `Duplicate value detected for field "${fieldName}". Please use a unique value.`
          errorColumn.value = fieldName // Highlight the column in the add row
        } else {
          errorMessage.value = 'Duplicate value detected. Please check unique fields.'
        }
      }

      // Optionally use toast for errors too
      toast({ title: 'Save Error', description: errorMessage.value, variant: 'destructive' })
    } finally {
      isSaving.value = false
    }
  }

  // Start editing a cell
  function startEditingCell(rowIndex: number, header: string, currentValue: any) {
    if (isSaving.value) return // Don't allow edit while saving
    if (['_id', 'created_at', 'updated_at'].includes(header)) return // Non-editable fields

    // Commit previous edit if any
    if (editingCell.value) {
      // Consider auto-saving on blur/new cell click, or require explicit save
      // For simplicity here, we just cancel the previous edit visually
      // await saveEdit(); // Uncomment if you want to auto-save previous cell
    }

    const doc = paginatedDocuments.value[rowIndex]
    if (!doc) return

    editingCell.value = { rowIndex, header }

    // Format value for editing input
    const schemaInfo = getSchemaInfo(header)
    const bsonType = Array.isArray(schemaInfo.bsonType)
      ? schemaInfo.bsonType[0]
      : schemaInfo.bsonType

    if (bsonType === 'bool') {
      editValue.value = !!currentValue // Ensure boolean
    } else if (bsonType === 'date') {
      // Ensure date is in YYYY-MM-DDTHH:mm format for datetime-local input
      editValue.value = currentValue ? new Date(currentValue).toISOString().slice(0, 16) : ''
    } else if (isReferenceField(header)) {
      editValue.value = currentValue || '' // Store the ID for the select
      // Ensure options are loaded for the reference field
      const refCollection = getReferencedCollection(header)
      if (refCollection) {
        fetchReferenceOptions(refCollection)
      }
    } else if (typeof currentValue === 'object' && currentValue !== null) {
      editValue.value = JSON.stringify(currentValue, null, 2) // Pretty print objects/arrays
    } else {
      editValue.value = String(currentValue ?? '') // Handle null/undefined
    }
  }

  // Cancel editing a cell
  function cancelEdit() {
    editingCell.value = null
    editValue.value = ''
  }

  function updateDocument(docId: string, updatedDoc: any) {
    // Find document index in the main documents array
    const docIndex = documents.value.findIndex((doc) => doc._id.$oid === docId)

    if (docIndex !== -1) {
      // Replace the document with the updated version
      documents.value = documents.value.map((doc, index) => (index === docIndex ? updatedDoc : doc))

      // No need to update paginatedDocuments as it's a computed property
      // that will automatically update based on the documents array
    } else {
      // Document with ID not found in local state
    }
  }

  // Save an edited cell value
  async function saveEdit() {
    if (!editingCell.value || isSaving.value) return

    const { rowIndex, header } = editingCell.value
    const doc = paginatedDocuments.value[rowIndex]
    if (!doc) {
      cancelEdit()
      return
    }

    const docId = doc._id.$oid
    isSaving.value = true
    errorMessage.value = ''
    const originalValue = doc[header] // Store original value for comparison/revert

    try {
      let valueToSave: any
      const schemaInfo = getSchemaInfo(header)
      const bsonType = Array.isArray(schemaInfo.bsonType)
        ? schemaInfo.bsonType[0]
        : schemaInfo.bsonType

      // --- Parse/Validate editValue based on type ---
      if (isReferenceField(header)) {
        valueToSave = editValue.value // Assume editValue holds the selected ID string
      } else if (bsonType === 'bool') {
        valueToSave = Boolean(editValue.value)
      } else if (bsonType === 'date') {
        valueToSave = editValue.value ? new Date(editValue.value).toISOString() : null
      } else if (['int', 'long'].includes(bsonType)) {
        valueToSave = parseInt(editValue.value, 10)
        if (isNaN(valueToSave)) throw new Error('Invalid integer value')
      } else if (['double', 'decimal'].includes(bsonType)) {
        valueToSave = parseFloat(editValue.value)
        if (isNaN(valueToSave)) throw new Error('Invalid number value')
      } else if (bsonType === 'string') {
        valueToSave = editValue.value // Keep as string
      } else if (bsonType === 'object' || bsonType === 'array') {
        try {
          valueToSave = JSON.parse(editValue.value)
        } catch (e) {
          throw new Error('Invalid JSON format')
        }
      } else {
        valueToSave = editValue.value // Default case
      }

      // --- Check if value actually changed ---
      // Note: Deep comparison might be needed for objects/arrays if stringify isn't sufficient
      if (JSON.stringify(valueToSave) === JSON.stringify(originalValue)) {
        cancelEdit() // Exit edit mode
        return
      }

      const update = { [header]: valueToSave }
      const response = await fetch(
        `${API_BASE}/collections/${collectionName.value}/documents/${docId}`,
        {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(update),
        }
      )

      if (!response.ok) {
        let errorData
        try {
          errorData = await response.json()
        } catch {
          /* ignore */
        }
        throw new Error(errorData?.error || `Update failed: HTTP ${response.status}`)
      }

      const result = await response.json()

      if (result.success && result.data?.document) {
        const updatedDoc = result.data.document

        // Update the document in the store - this will automatically update any computed properties
        // using the proper store action instead of direct mutation
        updateDocument(docId, updatedDoc)

        toast({ title: 'Update Successful', description: `Field '${header}' updated.` })
        cancelEdit() // Exit edit mode on success
      } else if (result.success && result.data?.modified_count === 0) {
        // API succeeded but didn't modify (e.g., value was the same)
        cancelEdit()
      } else {
        throw new Error(result.error || 'Update failed (API indicated failure)')
      }
    } catch (err: any) {
      errorMessage.value = `Error updating field '${header}': ${err.message}`
      toast({ title: 'Update Error', description: errorMessage.value, variant: 'destructive' })
    } finally {
      isSaving.value = false
    }
  }

  // Delete a document
  async function deleteDocument(docId: string) {
    if (!collectionName.value) return
    pendingDeleteId.value = docId // Style the row during delete attempt
    errorMessage.value = ''

    try {
      const response = await fetch(
        `${API_BASE}/collections/${collectionName.value}/documents/${docId}`,
        { method: 'DELETE' }
      )

      if (!response.ok) {
        let errorData
        try {
          errorData = await response.json()
        } catch {
          /* ignore */
        }
        throw new Error(errorData?.error || `Delete failed: HTTP ${response.status}`)
      }

      // Check response body if API confirms success
      const result = await response.json() // Or handle non-JSON responses if applicable

      if (result.success) {
        // Adjust based on your API response structure
        toast({ title: 'Document Deleted', description: `Document ID: ${docId} deleted.` })
        // Remove from local state *after* successful deletion
        documents.value = documents.value.filter((doc) => doc._id.$oid !== docId)
        selectedRows.value.delete(docId) // Remove from selection if present
      } else {
        throw new Error(result.error || 'Delete failed (API indicated failure)')
      }
    } catch (err: any) {
      errorMessage.value = `Error deleting document ${docId}: ${err.message}`
      toast({ title: 'Delete Error', description: errorMessage.value, variant: 'destructive' })
    } finally {
      // Clear pending style regardless of success/failure *after* potential fetchDocuments
      // If fetchDocuments is called on success, it resets loading and pending ID
      if (errorMessage.value) {
        // Only clear if there was an error (success clears via fetch)
        pendingDeleteId.value = null
      }
      // Consider calling fetchDocuments() here instead of filtering locally
      // await fetchDocuments(); // This ensures consistency with the backend
    }
  }

  // Toggle row selection
  function toggleRow(id: string) {
    const newSet = new Set(selectedRows.value)
    if (newSet.has(id)) {
      newSet.delete(id)
    } else {
      newSet.add(id)
    }
    selectedRows.value = newSet // Assign new Set to trigger reactivity
    // Optional toast notification removed for brevity, add back if desired
  }

  // Reset selection
  function resetSelection() {
    selectedRows.value = new Set()
    editingCell.value = null // Also cancel any active edit
  }

  // Set current page
  function setPage(page: number) {
    const newPage = Math.max(1, Math.min(page, totalPages.value))
    if (currentPage.value !== newPage) {
      currentPage.value = newPage
      // Fetching might not be needed if pagination is purely client-side
      // If API supports server-side pagination, call fetchDocuments() here
    }
  }

  // Set page size
  function setPageSize(size: number) {
    pageSize.value = size
    currentPage.value = 1 // Reset to page 1 when size changes
    // Fetching might not be needed if pagination is purely client-side
    // If API supports server-side pagination, call fetchDocuments() here
  }

  async function updateDocumentField(documentId: string, field: string, value: any) {
    if (!collectionName.value) return

    errorMessage.value = ''

    try {
      const update = { [field]: value }
      const response = await fetch(
        `${API_BASE}/collections/${collectionName.value}/documents/${documentId}`,
        {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(update),
        }
      )

      if (!response.ok) {
        let errorData
        try {
          errorData = await response.json()
        } catch {
          /* ignore */
        }
        throw new Error(errorData?.error || `Update failed: HTTP ${response.status}`)
      }

      const result = await response.json()

      if (result.success) {
        // Update local document state
        const docIndex = documents.value.findIndex((d) => d._id.$oid === documentId)
        if (docIndex !== -1) {
          documents.value[docIndex] = { ...documents.value[docIndex], ...update }
          // Force reactivity update
          documents.value = [...documents.value]
        }
        return true
      } else {
        throw new Error(result.error || 'Update failed (API indicated failure)')
      }
    } catch (err: any) {
      errorMessage.value = `Error updating field '${field}': ${err.message}`
      toast({ title: 'Update Error', description: errorMessage.value, variant: 'destructive' })
      return false
    }
  }

  async function updateUIMetadata(uiUpdate: Record<string, any>) {
    if (!collectionName.value) return

    if (previewMode.value) {
      // Preview mode: save to sessionStorage
      const previewStateKey = `previewState-${collectionName.value}`
      const currentPreviewState = JSON.parse(sessionStorage.getItem(previewStateKey) || '{}')
      const updatedPreviewState = { ...currentPreviewState, ...uiUpdate }
      sessionStorage.setItem(previewStateKey, JSON.stringify(updatedPreviewState))

      // Update local schema without API call
      collectionSchema.value.ui = {
        ...collectionSchema.value.ui,
        ...uiUpdate,
      }
      return
    }

    // Regular mode: save to backend
    try {
      const response = await fetch(`${API_BASE}/collections/${collectionName.value}/ui-metadata`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(uiUpdate),
      })

      if (!response.ok) throw new Error('Failed to update UI metadata')
      await fetchSchema() // Refresh schema to get updated UI metadata
    } catch (error) {
      throw error
    }
  }

  const hiddenColumns = ref<string[]>([])

  // Add getter for visible headers
  const visibleHeaders = computed(() => {
    return tableHeaders.value.filter((header) => !hiddenColumns.value.includes(header))
  })

  // Add action to toggle visibility
  function toggleColumnVisibility(header: string) {
    const index = hiddenColumns.value.indexOf(header)
    if (index === -1) {
      hiddenColumns.value.push(header)
    } else {
      hiddenColumns.value.splice(index, 1)
    }

    if (previewMode.value) {
      savePreviewState()
    } else {
      saveColumnVisibilityToBackend()
    }
  }

  // Add action to save to backend
  async function saveColumnVisibilityToBackend() {
    if (!collectionName.value) return
    try {
      await updateUIMetadata({
        ...(collectionSchema.value.ui || {}), // Preserve existing UI metadata
        hiddenColumns: hiddenColumns.value, // Override hiddenColumns
      })
    } catch (error) {
      // Error handling without console.log
    }
  }

  // Update and save column widths
  async function updateColumnWidth(header: string, width: number) {
    if (!collectionSchema.value.ui) {
      collectionSchema.value.ui = {}
    }
    if (!collectionSchema.value.ui.columnWidths) {
      collectionSchema.value.ui.columnWidths = {}
    }
    // Update local schema immediately for responsiveness
    collectionSchema.value.ui.columnWidths = {
      ...collectionSchema.value.ui.columnWidths,
      [header]: Math.max(50, width), // Ensure minimum width
    }
    // Debounced save to backend will be handled in the component using this action
    await saveColumnWidthsToBackend()
  }

  // Reset a specific column width
  async function resetColumnWidth(header: string) {
    if (collectionSchema.value?.ui?.columnWidths?.[header]) {
      const newWidths = { ...collectionSchema.value.ui.columnWidths }
      delete newWidths[header]
      collectionSchema.value.ui.columnWidths = newWidths
      await saveColumnWidthsToBackend()
    }
  }

  const savePreviewState = useDebounceFn(() => {
    if (!collectionName.value) return

    // Save UI settings
    sessionStorage.setItem(
      `previewState-${collectionName.value}`,
      JSON.stringify(collectionSchema.value.ui)
    )

    // Save hidden columns
    sessionStorage.setItem(
      `previewHidden-${collectionName.value}`,
      JSON.stringify(hiddenColumns.value)
    )
  }, 500)

  // Save column widths to backend (called by debounced function in component)
  async function saveColumnWidthsToBackend() {
    if (!collectionName.value) return

    if (previewMode.value) {
      // Save to sessionStorage for Preview Mode
      const previewState = {
        ...collectionSchema.value.ui,
        columnWidths: columnWidths.value,
      }
      sessionStorage.setItem(`previewState-${collectionName.value}`, JSON.stringify(previewState))
    } else {
      try {
        const response = await fetch(
          `${API_BASE}/collections/${collectionName.value}/ui-metadata`,
          {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ columnWidths: columnWidths.value }),
          }
        )

        if (!response.ok) {
          let errorData
          try {
            errorData = await response.json()
          } catch {
            /* ignore */
          }
          throw new Error(errorData?.error || `Save widths failed: HTTP ${response.status}`)
        }

        const result = await response.json()
        if (!result.success) {
          throw new Error(result.error || 'Failed to save column widths (API Error)')
        }
      } catch (err: any) {
        toast({
          title: 'Save Error',
          description: `Could not save column widths: ${err.message}`,
          variant: 'destructive',
        })
      }
    }
  }

  // Clear error message
  function clearError() {
    errorMessage.value = ''
  }

  const shortNames = computed(() => {
    return collectionSchema.value?.ui?.short_names || {}
  })

  // Return state, getters, and actions
  return {
    // State
    hasMore,
    collectionName,
    documents,
    collectionSchema,
    isLoading,
    errorMessage,
    pageSize,
    currentPage,
    filterQuery, // Keep filter local to component or move here if needed globally
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
    collectionsList,
    errorColumn,
    addingRowError,
    hiddenColumns,
    visibleHeaders,
    previewMode,

    // Getters
    totalDocuments,
    totalPages,
    paginatedDocuments,
    tableHeaders,
    columnWidths,
    allSelected,

    shortNames,
    // Actions
    fetchCollections,
    setCollection,
    fetchSchema, // Expose if needed externally, otherwise internal use
    fetchDocuments,
    fetchReferenceOptions,
    getReferenceLabel,
    initializeNewDocument, // Might be internal unless needed outside
    startAdding,
    cancelAdding,
    saveNewDocument,
    startEditingCell,
    cancelEdit,
    saveEdit,
    updateDocument,
    deleteDocument,
    toggleRow,
    resetSelection,
    changeView,
    loadNextPage,
    setPage,
    setPageSize,
    updateDocumentField,
    updateUIMetadata,
    toggleColumnVisibility,
    saveColumnVisibilityToBackend,
    updateColumnWidth,
    resetColumnWidth,
    saveColumnWidthsToBackend, // Exposed for debouncing in component
    clearError,
    getSchemaInfo, // Expose helpers if needed in component template
    isReferenceField,
    getReferencedCollection,
    pinDocument,
    unpinDocument,
  }
})
