<!-- src/views/CSVImportView.vue -->
<script setup lang="ts">
  import { ref, computed, onMounted, onUnmounted, watch } from 'vue' // [cite: 1]
  import { useRoute } from 'vue-router' // [cite: 1]
  import { useToast } from '@/components/ui/toast' // [cite: 1]
  import { Button } from '@/components/ui/button' // [cite: 1]
  import { storeToRefs } from 'pinia' // [cite: 1]
  import MongoDBDataTable from '@/components/MongoDBDataTable.vue' // [cite: 1]
  import { parseCSV } from '@/utils/parseCSV' // [cite: 1]
  import { useDataTableStore } from '@/store/dataTableStore' // [cite: 1]
  import { getApiBaseUrl } from '@/utils/api' // [cite: 1]

  // Import Dialog components
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogClose, // Import DialogClose for explicit closing
  } from '@/components/ui/dialog'

  const route = useRoute() // [cite: 2]
  const { toast } = useToast() // [cite: 2]
  const dataTableStore = useDataTableStore() // [cite: 2]
  const { collectionSchema } = storeToRefs(dataTableStore) // [cite: 2]
  const collectionName = computed(() => route.params.name as string) // [cite: 2]
  const primaryKey = computed(() => {
    // [cite: 2]
    return collectionSchema.value?.primaryKey // [cite: 2]
  }) // [cite: 2]

  // Debug flag to track pagination behavior
  const DEBUG = true // [cite: 2]
  const logDebug = (...args: any[]) => {
    // [cite: 2]
    if (DEBUG) console.debug('[CSVImport]', ...args) // [cite: 2]
  } // [cite: 2]

  // Update data structure to handle pagination
  const dataDisplayMode = ref<'valid' | 'invalid'>('valid') // [cite: 3]

  const validData = ref({
    // [cite: 3]
    data: [] as any[], // [cite: 3]
    currentPage: 1, // [cite: 3]
    totalPages: 1, // [cite: 3]
    isLoading: false, // [cite: 3]
    total: 0, // Track total record count // [cite: 3]
  }) // [cite: 3]

  const invalidData = ref({
    // [cite: 3]
    data: [] as any[], // [cite: 3]
    currentPage: 1, // [cite: 3]
    totalPages: 1, // [cite: 3]
    isLoading: false, // [cite: 3]
    total: 0, // Track total record count // [cite: 3]
  }) // [cite: 3]

  // Update preview data based on mode
  const previewData = computed(
    () =>
      // [cite: 3]
      dataDisplayMode.value === 'valid' ? validData.value.data : invalidData.value.data // [cite: 4]
  )

  // Compute if there's more data to load
  const hasMore = computed(() => {
    // [cite: 4]
    const mode = dataDisplayMode.value // [cite: 4]
    const result = // [cite: 4]
      mode === 'valid' // [cite: 4]
        ? validData.value.currentPage < validData.value.totalPages // [cite: 4]
        : invalidData.value.currentPage < invalidData.value.totalPages // [cite: 4]

    logDebug('hasMore computed:', {
      // [cite: 4]
      mode, // [cite: 4]
      currentPage: mode === 'valid' ? validData.value.currentPage : invalidData.value.currentPage, // [cite: 4]
      totalPages: mode === 'valid' ? validData.value.totalPages : invalidData.value.totalPages, // [cite: 5]
      result, // [cite: 5]
    }) // [cite: 5]
    return result // [cite: 5]
  }) // [cite: 5]

  const fileInput = ref<HTMLInputElement | null>(null) // [cite: 6]
  const hasImportedData = ref(false) // [cite: 6]

  // --- NEW Refs for Deep Validation ---
  const showValidationDialog = ref(false)
  const isDeepValidating = ref(false)
  const validationSummary = ref<{
    validated_count: number
    conflicts_found: number
    remaining_valid: number
  } | null>(null)

  // Watch for dataDisplayMode changes to log state transitions
  watch(dataDisplayMode, (newMode, oldMode) => {
    // [cite: 6]
    logDebug('Display mode changed:', { from: oldMode, to: newMode, hasMore: hasMore.value }) // [cite: 6]
  }) // [cite: 6]

  // Helper function to normalize document ID regardless of format
  const getNormalizedId = (item: any): string | null => {
    // [cite: 7]
    return item?._id?.$oid || (typeof item?._id === 'string' ? item._id : null) || item?.id || null // [cite: 8]
  }

  // Function to refresh data from temp storage (e.g., after validation)
  const refreshTempData = async (page = 1) => {
    logDebug(`Refreshing temp data for page ${page}`)
    const dataRef = dataDisplayMode.value === 'valid' ? validData : invalidData
    dataRef.value.isLoading = true
    try {
      const url = `${getApiBaseUrl()}/api/csv-temp/${collectionName.value}?valid_page=${page}&valid_page_size=20&invalid_page=${page}&invalid_page_size=20` // Fetch first page for both
      const response = await fetch(url)
      if (!response.ok) {
        const errorText = await response.text()
        throw new Error(`Failed to load temp data: ${response.status} - ${errorText}`)
      }
      const responseData = await response.json()
      const { valid, invalid } = responseData

      // Reset data before assigning new page
      validData.value.data = valid.data || []
      validData.value.currentPage = valid.page || 1
      validData.value.totalPages = Math.ceil(valid.total / valid.page_size) || 1
      validData.value.total = valid.total || 0

      invalidData.value.data = invalid.data || []
      invalidData.value.currentPage = invalid.page || 1
      invalidData.value.totalPages = Math.ceil(invalid.total / invalid.page_size) || 1
      invalidData.value.total = invalid.total || 0

      hasImportedData.value = validData.value.total > 0 || invalidData.value.total > 0 // Update based on totals

      logDebug('Temp data refreshed successfully', {
        valid: { count: validData.value.data.length, total: validData.value.total },
        invalid: { count: invalidData.value.data.length, total: invalidData.value.total },
      })

      // Show toast for current mode
      toast({
        title: 'Data Loaded',
        description: `${dataDisplayMode.value} data fetched successfully.`,
      })
    } catch (error: any) {
      console.error('Refresh error:', error)
      toast({ title: 'Refresh Error', description: error.message, variant: 'destructive' })
    } finally {
      dataRef.value.isLoading = false
    }
  }

  // Load more data when button is clicked
  const loadMoreData = async () => {
    const mode = dataDisplayMode.value
    logDebug('loadMoreData triggered for mode:', mode)

    const dataRef = mode === 'valid' ? validData : invalidData
    logDebug('Current pagination state:', {
      page: dataRef.value.currentPage,
      totalPages: dataRef.value.totalPages,
      hasMore: hasMore.value,
      isLoading: dataRef.value.isLoading,
      dataLength: dataRef.value.data.length,
      total: dataRef.value.total,
    })

    if (dataRef.value.isLoading || !hasMore.value) {
      logDebug('Aborting loadMore - loading:', dataRef.value.isLoading, 'hasMore:', hasMore.value)
      return
    }

    dataRef.value.isLoading = true
    try {
      const nextPage = dataRef.value.currentPage + 1
      logDebug('Attempting to load page:', nextPage)

      const validPage = mode === 'valid' ? nextPage : validData.value.currentPage
      const invalidPage = mode === 'invalid' ? nextPage : invalidData.value.currentPage

      const url =
        `${getApiBaseUrl()}/api/csv-temp/${collectionName.value}?` +
        `valid_page=${validPage}&valid_page_size=20&` +
        `invalid_page=${invalidPage}&invalid_page_size=20`

      logDebug('Fetching from URL:', url)

      const response = await fetch(url)
      if (!response.ok) {
        const errorText = await response.text()
        logDebug('Load failed - status:', response.status, 'response:', errorText)
        throw new Error(`Failed to load data: ${response.status}`)
      }

      const responseData = await response.json()
      logDebug('API response structure:', Object.keys(responseData))
      const { valid, invalid } = responseData
      logDebug('API response data:', {
        valid: {
          page: valid.page,
          total: valid.total,
          page_size: valid.page_size,
          dataLength: valid.data?.length,
        },
        invalid: {
          page: invalid.page,
          total: invalid.total,
          page_size: invalid.page_size,
          dataLength: invalid.data?.length,
        },
      })

      if (mode === 'valid') {
        // Check if we actually got new data
        if (!valid.data || valid.data.length === 0) {
          logDebug('Warning: No new valid data received for page', nextPage)
        }

        logDebug('Valid data before update:', validData.value.data.length)

        // Create set of existing IDs with normalized format for robust comparison
        const existingIds = new Set(
          validData.value.data.map(getNormalizedId).filter(Boolean) // Filter out any null IDs
        )

        // Filter out items that already exist by checking normalized IDs
        const newItems = valid.data.filter((item: any) => {
          const normalizedId = getNormalizedId(item)
          return normalizedId && !existingIds.has(normalizedId)
        })

        validData.value.data = [...validData.value.data, ...newItems]
        validData.value.currentPage = valid.page
        validData.value.totalPages = Math.ceil(valid.total / valid.page_size)
        validData.value.total = valid.total
        logDebug('Valid data after update:', {
          length: validData.value.data.length,
          newItemsAdded: newItems.length,
          currentPage: validData.value.currentPage,
          totalPages: validData.value.totalPages,
        })
      } else {
        // Check if we actually got new data
        if (!invalid.data || invalid.data.length === 0) {
          logDebug('Warning: No new invalid data received for page', nextPage)
        }

        logDebug('Invalid data before update:', invalidData.value.data.length)
        // Use _id to check for duplicates
        const existingInvalidIds = new Set(
          invalidData.value.data.map(getNormalizedId).filter(Boolean)
        )
        const newItems = invalid.data.filter((item: any) => {
          const normalizedId = getNormalizedId(item)
          return normalizedId && !existingInvalidIds.has(normalizedId)
        })

        invalidData.value.data = [...invalidData.value.data, ...newItems]
        invalidData.value.currentPage = invalid.page
        invalidData.value.totalPages = Math.ceil(invalid.total / invalid.page_size)
        invalidData.value.total = invalid.total
        logDebug('Invalid data after update:', {
          length: invalidData.value.data.length,
          newItemsAdded: newItems.length,
          currentPage: invalidData.value.currentPage,
          totalPages: invalidData.value.totalPages,
        })
      }

      logDebug('Updated pagination state:', {
        valid: {
          page: validData.value.currentPage,
          totalPages: validData.value.totalPages,
          hasMore: validData.value.currentPage < validData.value.totalPages,
        },
        invalid: {
          page: invalidData.value.currentPage,
          totalPages: invalidData.value.totalPages,
          hasMore: invalidData.value.currentPage < invalidData.value.totalPages,
        },
      })

      // Show toast for loaded more data
      toast({
        title: 'More Data Loaded',
        description: `Successfully loaded more ${mode} data.`,
        variant: 'success',
      })
    } catch (error: any) {
      console.error('Load error:', error)
      toast({ title: 'Load Error', description: error.message, variant: 'destructive' })
    } finally {
      dataRef.value.isLoading = false
      logDebug('Load completed for mode:', mode)
    }
  }

  const handleFileUpload = async (event: Event) => {
    // [cite: 23]
    const file = (event.target as HTMLInputElement).files?.[0] // [cite: 23]
    if (!file) return // [cite: 24]

    logDebug('Upload started:', { fileName: file.name, size: file.size }) // [cite: 24]
    // Reset previous state if any
    resetImport(true) // Pass silent=true to avoid toast on reset before upload

    try {
      // [cite: 24]
      const { data: csvData } = await parseCSV(file) // [cite: 24]
      logDebug('CSV parsed successfully:', { rowCount: csvData.length }) // [cite: 24]

      const schema = collectionSchema.value.properties || {} // [cite: 25]

      // Create short name map for field mapping
      const shortNameMap = createShortNameMap(schema) // [cite: 25]
      logDebug('Field mapping:', shortNameMap) // [cite: 25]

      // Check for required columns
      const csvHeaders = csvData.length > 0 ? Object.keys(csvData[0]) : [] // [cite: 26]
      logDebug('CSV headers:', csvHeaders) // [cite: 26]

      // --- Primary Key Check (using schema's primaryKey) ---
      const schemaPrimaryKey = primaryKey.value // Get primary key from schema [cite: 2]
      const primaryKeyHeader = schemaPrimaryKey
        ? shortNameMap[schemaPrimaryKey] || schemaPrimaryKey
        : null

      if (schemaPrimaryKey && primaryKeyHeader && !csvHeaders.includes(primaryKeyHeader)) {
        logDebug('Missing required primary key column:', primaryKeyHeader) // [cite: 27]
        toast({
          // [cite: 28]
          title: 'Invalid CSV Format', // [cite: 28]
          description: `Missing primary key column: ${primaryKeyHeader} (expected from schema field: ${schemaPrimaryKey})`, // [cite: 28]
          variant: 'destructive', // [cite: 28]
        })
        return // [cite: 29]
      }
      // --- End Primary Key Check ---

      // Validate CSV data and split into valid/invalid
      logDebug('Validating CSV data...') // [cite: 29]
      const { valid: validCsvData, invalid: invalidCsvData } = validateCSVData(csvData, schema) // [cite: 29]
      logDebug('Validation complete:', {
        // [cite: 29]
        validCount: validCsvData.length, // [cite: 29]
        invalidCount: invalidCsvData.length, // [cite: 29]
      }) // [cite: 29]

      // Show upload counts in toast
      const validCount = validCsvData.length // [cite: 29]
      const invalidCount = invalidCsvData.length // [cite: 30]
      toast({
        // [cite: 30]
        title: `Initial CSV Scan Results`, // Changed title
        description: `Potential Valid: ${validCount}, Invalid: ${invalidCount}. Click 'Validate & Continue' for final checks.`, // Updated description
        duration: 7000, // [cite: 30] Longer duration
      }) // [cite: 30]

      // Transform valid data for storage
      logDebug('Transforming valid data...') // [cite: 30]
      const transformedValid = transformCSVData(validCsvData, shortNameMap, collectionSchema.value) // [cite: 30]
      logDebug('Valid data transformed:', { count: transformedValid.length }) // [cite: 30]

      // Transform invalid data to include all CSV columns
      logDebug('Transforming invalid data...') // [cite: 31]
      const transformedInvalid = invalidCsvData.map(({ row, errors }) => {
        // [cite: 31]
        const transformedRow: Record<string, string> = {} // [cite: 31]

        // Use CSV headers to include all columns from the original CSV
        csvHeaders.forEach((header) => {
          // [cite: 31]
          const value = row[header] // [cite: 31]
          transformedRow[header] = value !== undefined ? String(value) : '' // [cite: 32]
        }) // [cite: 32]

        return {
          // [cite: 32]
          ...transformedRow, // [cite: 32]
          errors, // [cite: 32]
        } // [cite: 32]
      }) // [cite: 32]
      logDebug('Invalid data transformed:', { count: transformedInvalid.length }) // [cite: 32]

      // Send to backend temp storage
      logDebug('Sending initial data to backend temp storage...') // [cite: 32]
      const response = await fetch(`${getApiBaseUrl()}/api/csv-temp/${collectionName.value}`, {
        // [cite: 32]
        method: 'POST', // [cite: 33]
        headers: { 'Content-Type': 'application/json' }, // [cite: 33]
        body: JSON.stringify({ valid: transformedValid, invalid: transformedInvalid }), // [cite: 33]
      }) // [cite: 33]

      if (!response.ok) {
        // [cite: 33]
        const errorText = await response.text() // [cite: 33]
        logDebug('Backend temp save failed:', { status: response.status, response: errorText }) // [cite: 33]
        throw new Error(`Failed to save temporary data: ${response.status}`) // [cite: 33]
      }

      logDebug('Initial data saved to backend temp storage successfully') // [cite: 34]

      // --- Don't immediately populate local data refs ---
      // Instead, set flag to show dialog
      hasImportedData.value = true // Still set this to change the view
      showValidationDialog.value = true // <<< NEW: Trigger the dialog
      validationSummary.value = null // Reset previous summary

      // --- Removed local data population and selection logic ---
      // validData.value.data = transformedValid // [cite: 34] // REMOVED
      // invalidData.value.data = transformedInvalid // [cite: 34] // REMOVED
      // ... pagination setup removed ...
      // ... selection setup removed ... [cite: 36, 37]

      //   toast({ title: 'CSV Processed', description: 'Ready for final validation.' }) // Updated toast [cite: 37]
    } catch (error: any) {
      // [cite: 37]
      // Handle parsing/validation errors
      console.error('Import failed:', error) // [cite: 37]
      toast({ title: 'Import Failed', description: error.message, variant: 'destructive' }) // [cite: 37]
      if (fileInput.value) fileInput.value.value = '' // Reset file input on error
    }
  }

  // --- NEW: Function to trigger deep validation ---
  const triggerDeepValidation = async () => {
    logDebug('Deep validation triggered')
    isDeepValidating.value = true
    showValidationDialog.value = false // Close dialog
    validationSummary.value = null

    try {
      const response = await fetch(`${getApiBaseUrl()}/api/csv-validate/${collectionName.value}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        // No body needed, server uses collection name to find temp data
      })

      if (!response.ok) {
        let errorMsg = `Validation request failed: ${response.status}`
        try {
          const errorData = await response.json()
          errorMsg = errorData.error || errorMsg
        } catch {
          /* Ignore if body is not JSON */
        }
        throw new Error(errorMsg)
      }

      const result = await response.json()
      if (result.success && result.data) {
        validationSummary.value = result.data // Store summary
        logDebug('Deep validation successful:', result.data)
        toast({
          title: 'Validation Complete',
          description: `Checked: ${result.data.validated_count}, Conflicts: ${result.data.conflicts_found}, Ready to Import: ${result.data.remaining_valid}`,
          duration: 7000,
        })
        // Refresh the data displayed in the tables
        await refreshTempData()
      } else {
        throw new Error(result.error || 'Validation completed with an unknown error.')
      }
    } catch (error: any) {
      console.error('Deep validation failed:', error)
      toast({ title: 'Validation Failed', description: error.message, variant: 'destructive' })
      // Optionally reset state or guide user?
    } finally {
      isDeepValidating.value = false
    }
  }

  // Helper functions (Keep validateCSVData, createShortNameMap, convertValueBySchema, transformCSVData, generateMongoObjectId, randomHexString as they are)
  const validateCSVData = (
    // [cite: 37]
    data: any[], // [cite: 37]
    schema: Record<string, any> // [cite: 37]
  ): { valid: any[]; invalid: Array<{ row: any; errors: string[] }> } => {
    // [cite: 38]
    const valid: any[] = [] // [cite: 38]
    const invalid: Array<{ row: any; errors: string[] }> = [] // [cite: 39]

    data.forEach((row) => {
      // [cite: 39]
      const errors: string[] = [] // [cite: 39]

      // Primary key field check (using schema primary key)
      const schemaPrimaryKey = primaryKey.value // Get primary key from schema
      if (schemaPrimaryKey) {
        // Find the corresponding header name in the CSV data (could be original or short name)
        const pkHeader = Object.keys(row).find(
          (header) =>
            header === schemaPrimaryKey ||
            header === collectionSchema.value?.ui?.short_names?.[schemaPrimaryKey]
        )
        const value = pkHeader ? row[pkHeader] : undefined
        if (value === null || value === undefined || value === '') {
          // [cite: 39]
          errors.push(`Missing primary key field: ${schemaPrimaryKey}`) // [cite: 39]
        }
      }

      // Type validation
      Object.entries(row).forEach(([field, value]) => {
        // [cite: 40]
        const fieldSchema = schema[field] // [cite: 40]
        if (!fieldSchema || value === null || value === '') return // [cite: 40]

        const type = Array.isArray(fieldSchema.bsonType) // [cite: 40]
          ? fieldSchema.bsonType[0] // [cite: 40]
          : fieldSchema.bsonType // [cite: 40]

        if (
          // [cite: 40]
          (type === 'number' || type === 'int' || type === 'double' || type === 'long') && // [cite: 41]
          isNaN(Number(value)) // [cite: 41]
        ) {
          // [cite: 41]
          errors.push(`Invalid number in ${field}`) // [cite: 41]
        } // [cite: 41]

        if (
          // [cite: 41]
          type === 'bool' && // [cite: 41]
          !['true', 'false', '0', '1', 'yes', 'no', ''].includes(String(value).toLowerCase()) // [cite: 41]
        ) {
          // [cite: 41]
          errors.push(`Invalid boolean value in ${field}`) // [cite: 42]
        } // [cite: 42]

        if (type === 'date' && isNaN(Date.parse(String(value)))) {
          // [cite: 42]
          errors.push(`Invalid date in ${field}`) // [cite: 42]
        } // [cite: 42]
      }) // [cite: 42]

      if (errors.length > 0) {
        // [cite: 42]
        invalid.push({ row, errors }) // [cite: 42]
      } else {
        // [cite: 42]
        valid.push(row) // [cite: 42]
      } // [cite: 43]
    }) // [cite: 43]

    return { valid, invalid } // [cite: 43]
  } // [cite: 43]

  const createShortNameMap = (schema: Record<string, any>) => {
    // [cite: 43]
    return Object.keys(schema).reduce(
      // [cite: 43]
      (acc, field) => {
        // [cite: 43]
        acc[field] = schema[field]?.ui?.short_name || field // [cite: 44]
        return acc // [cite: 44]
      }, // [cite: 44]
      {} as Record<string, string> // [cite: 44]
    ) // [cite: 44]
  } // [cite: 44]

  const convertValueBySchema = (value: any, fieldSchema: any) => {
    // [cite: 44]
    if (value === null || value === undefined || value === '') return null // [cite: 44]

    const type = Array.isArray(fieldSchema.bsonType) // [cite: 44]
      ? fieldSchema.bsonType[0] // [cite: 45]
      : fieldSchema.bsonType // [cite: 45]

    switch (
      type // [cite: 45]
    ) {
      case 'bool': // [cite: 45]
        if (value === '1' || value === 'yes' || String(value).toLowerCase() === 'true') return true // [cite: 45]
        if (value === '0' || value === 'no' || String(value).toLowerCase() === 'false') return false // [cite: 45]
        return null // [cite: 45]
      case 'int': // [cite: 45]
      case 'long': // [cite: 45]
        return parseInt(value, 10) // [cite: 45]
      case 'double': // [cite: 46]
      case 'number': // [cite: 46]
        return parseFloat(value) // [cite: 46]
      case 'date': // [cite: 46]
        try {
          // [cite: 46]
          return new Date(value).toISOString() // [cite: 46]
        } catch {
          // [cite: 46]
          return null // [cite: 46]
        } // [cite: 46]
      default: // [cite: 46]
        return value // [cite: 46]
    } // [cite: 46]
  } // [cite: 46]

  const transformCSVData = (data: any[], shortNameMap: Record<string, string>, schema: any) => {
    // [cite: 47]
    const schemaFields = Object.keys(schema.properties || {}) // [cite: 47]

    return data.map((row) => {
      // [cite: 7]
      const transformed: Record<string, any> = {} // [cite: 47]

      // Map all schema fields
      schemaFields.forEach((field) => {
        // [cite: 47]
        // Find the CSV column that corresponds to this schema field
        const csvKey = // [cite: 47]
          Object.entries(shortNameMap).find(([_, shortName]) => shortName === field)?.[0] || field // [cite: 48]

        const rawValue = row[csvKey] // [cite: 48]
        // Convert values based on schema type
        transformed[field] = schema.properties[field] // [cite: 48]
          ? convertValueBySchema(rawValue, schema.properties[field]) // [cite: 49]
          : rawValue === '' || rawValue === undefined // [cite: 50]
            ? null // [cite: 51]
            : rawValue // [cite: 51]
      }) // [cite: 51]

      // Add MongoDB-compatible ObjectID instead of numeric ID
      transformed._id = { $oid: generateMongoObjectId() } // [cite: 51]

      // Add timestamps if they're part of the schema
      const now = new Date().toISOString() // [cite: 51]
      if (schemaFields.includes('created_at')) {
        // [cite: 51]
        transformed.created_at = transformed.created_at || now // [cite: 52]
      } // [cite: 52]

      if (schemaFields.includes('updated_at')) {
        // [cite: 52]
        transformed.updated_at = transformed.updated_at || now // [cite: 53]
      } // [cite: 53]

      return transformed // [cite: 53]
    }) // [cite: 53]
  } // [cite: 53]

  // Function to generate MongoDB-compatible ObjectIDs
  function generateMongoObjectId(): string {
    // [cite: 53]
    const timestamp = Math.floor(new Date().getTime() / 1000) // [cite: 53]
      .toString(16) // [cite: 53]
      .padStart(8, '0') // [cite: 53]
    const machineId = randomHexString(6) // [cite: 53]
    const processId = randomHexString(4) // [cite: 53]
    const counter = randomHexString(6) // [cite: 53]

    return timestamp + machineId + processId + counter // [cite: 53]
  } // [cite: 53]

  // Helper function to generate random hex strings of specified length
  function randomHexString(length: number): string {
    // [cite: 54]
    let result = '' // [cite: 54]
    const characters = '0123456789abcdef' // [cite: 54]
    for (let i = 0; i < length; i++) {
      // [cite: 54]
      result += characters.charAt(Math.floor(Math.random() * characters.length)) // [cite: 54]
    } // [cite: 54]
    return result // [cite: 54]
  } // [cite: 54]

  onMounted(async () => {
    // [cite: 54]
    logDebug('Component mounted, fetching initial data') // [cite: 54]
    // Fetch existing temp data if any
    await refreshTempData()
  })

  const triggerFileSelect = () => {
    // [cite: 67]
    logDebug('File select triggered') // [cite: 67]
    fileInput.value?.click() // [cite: 67]
  }

  const resetImport = async (silent = false) => {
    // Add silent flag
    logDebug('Import reset requested') // [cite: 67]
    try {
      // [cite: 67]
      // Delete from backend
      await fetch(`${getApiBaseUrl()}/api/csv-temp/${collectionName.value}`, { method: 'DELETE' }) // [cite: 68]
      logDebug('Backend data deleted') // [cite: 68]

      validData.value = {
        // [cite: 68]
        data: [], // [cite: 68]
        currentPage: 1, // [cite: 68]
        totalPages: 1, // [cite: 68]
        isLoading: false, // [cite: 68]
        total: 0, // [cite: 68]
      } // [cite: 68]

      invalidData.value = {
        // [cite: 68]
        data: [], // [cite: 68]
        currentPage: 1, // [cite: 68]
        totalPages: 1, // [cite: 68]
        isLoading: false, // [cite: 68]
        total: 0, // [cite: 68]
      } // [cite: 69]

      hasImportedData.value = false // [cite: 69]
      showValidationDialog.value = false // Hide dialog on reset
      validationSummary.value = null // Clear summary
      // Clear selected rows when resetting
      dataTableStore.selectedRows = new Set() // [cite: 69]
      if (fileInput.value) {
        // [cite: 69]
        fileInput.value.value = '' // [cite: 69]
      }
      logDebug('Import reset complete') // [cite: 69]
      if (!silent) {
        // Only show toast if not silent
        toast({ title: 'Reset Complete', description: 'Temporary data cleared' }) // [cite: 69]
      }
    } catch (error: any) {
      // [cite: 69]
      console.error('Reset failed:', error) // [cite: 69]
      toast({ title: 'Reset Failed', description: error.message, variant: 'destructive' }) // [cite: 70]
    } // [cite: 70]
  }

  onUnmounted(() => {
    // [cite: 70]
    logDebug('Component unmounting, clearing selected rows') // [cite: 70]
    // Clear selected rows when component is unmounted
    dataTableStore.selectedRows = new Set() // [cite: 70]
  }) // [cite: 70]
</script>

<template>
  <div
    v-if="!hasImportedData"
    class="file-import-container"
  >
    <div class="mb-4">
      <router-link :to="`/collection/${collectionName}`">
        <Button
          variant="outline"
          size="sm"
        >
          Back
        </Button>
      </router-link>
    </div>
    <input
      ref="fileInput"
      type="file"
      accept=".csv"
      class="hidden"
      @change="handleFileUpload"
    />
    <div class="excel-inspired-import">
      <div class="import-header">
        <h2>Import CSV to {{ collectionName }}</h2>
        <p class="import-subtitle">Add data to your collection from a CSV file</p>
      </div>

      <div
        class="import-dropzone"
        @click="triggerFileSelect"
      >
        <div class="dropzone-content">
          <div class="excel-icon">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              width="48"
              height="48"
            >
              <path
                fill="none"
                d="M0 0h24v24H0z"
              />
              <path
                d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6zm1 5h-5V4h1v2h4v1zm-8 5h3v-1H7v1zm0 2h3v-1H7v1zm0 2h8v-1H7v1zm8-6v-1H7v1h8z"
                fill="#107C41"
              />
            </svg>
          </div>
          <p class="dropzone-text">
            <span class="font-bold">Select a CSV file</span> or drag and drop it here
          </p>
          <div class="mt-4"><Button class="primary-button">Browse Files</Button></div>
          <p class="text-xs text-gray-500 mt-2">
            Make sure your CSV has headers matching the collection schema
          </p>
        </div>
      </div>

      <div
        v-if="fileInput?.files?.[0]"
        class="selected-file"
      >
        <span class="file-name">{{ fileInput.files[0].name }}</span>
        <span class="file-size">{{ (fileInput.files[0].size / 1024).toFixed(1) }} KB</span>
      </div>
    </div>
  </div>

  <div v-else>
    <div
      v-if="isDeepValidating"
      class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity z-[100] flex items-center justify-center"
    >
      <div class="flex flex-col items-center text-white">
        <svg
          class="animate-spin h-10 w-10 mb-3"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle
            class="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            stroke-width="4"
          ></circle>
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          ></path>
        </svg>
        <p class="text-lg font-semibold">Validating against database...</p>
        <p>Checking for existing IDs and unique fields.</p>
      </div>
    </div>

    <Dialog
      :open="showValidationDialog"
      @update:open="(val) => (showValidationDialog = val)"
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Confirm Validation</DialogTitle>
          <DialogDescription>
            Initial scan complete. Ready to check the potentially valid
            <span class="font-semibold">{{ validData.total }}</span> rows against the '{{
              collectionName
            }}' collection in the database for existing IDs or unique field conflicts? This may take
            a moment for large datasets.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <DialogClose as-child>
            <Button
              variant="outline"
              @click="showValidationDialog = false"
              >Cancel</Button
            >
          </DialogClose>
          <Button
            @click="triggerDeepValidation"
            :disabled="isDeepValidating"
          >
            Validate & Continue
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <div class="csv-import-container">
      <!-- hidden for now -->
      <div
        v-if="DEBUG"
        class="debug-info bg-gray-100 p-2 mb-4 text-xs font-mono overflow-auto hidden"
      >
        <details open>
          <summary class="font-bold cursor-pointer">Debug Info ({{ dataDisplayMode }})</summary>
          <pre>
Mode: {{ dataDisplayMode }}
Valid data: {{ validData.data.length }} / {{ validData.total }} items (page {{
              validData.currentPage
            }}/{{ validData.totalPages }})
Invalid data: {{ invalidData.data.length }} / {{ invalidData.total }} items (page {{
              invalidData.currentPage
            }}/{{ invalidData.totalPages }}) Has more: {{ hasMore }}
Is Deep Validating: {{ isDeepValidating }}
Validation Summary: {{ validationSummary || 'N/A' }}
          </pre>
        </details>
      </div>
      <MongoDBDataTable
        :preview-data="previewData"
        :data-display-mode="dataDisplayMode"
        @update:data-display-mode="dataDisplayMode = $event"
        preview-mode
      />
      <div
        v-if="hasMore"
        class="show-more-container text-center w-full z-50 my-10"
      >
        <Button
          :disabled="
            (dataDisplayMode === 'valid' ? validData.isLoading : invalidData.isLoading) ||
            isDeepValidating
          "
          @click="loadMoreData"
          class="show-more-button"
        >
          <template
            v-if="dataDisplayMode === 'valid' ? validData.isLoading : invalidData.isLoading"
          >
            <svg
              class="animate-spin -ml-1 mr-2 h-4 w-4"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
              ></circle>
              <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            Loading...
          </template>
          <template v-else>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="mr-2"
            >
              <polyline points="7 13 12 18 17 13"></polyline>
              <polyline points="7 6 12 11 17 6"></polyline>
            </svg>
            Show More
          </template>
        </Button>

        <div class="text-xs text-gray-500 mt-2 text-center">
          Showing
          {{ dataDisplayMode === 'valid' ? validData.data.length : invalidData.data.length }} of
          {{ dataDisplayMode === 'valid' ? validData.total : invalidData.total }}
          {{ dataDisplayMode === 'valid' ? 'valid' : 'invalid' }} records
        </div>
      </div>
    </div>
  </div>

  <button
    v-if="hasImportedData"
    @click="resetImport()"
    class="fixed bottom-4 left-4 z-40 flex items-center px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-full shadow-lg transition-colors duration-200 ease-in-out"
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      class="mr-2"
    >
      <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
      <path d="M3 3v5h5" />
      <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
      <path d="M16 16h5v5" />
    </svg>
    Reset
  </button>
</template>

<style scoped>
  .csv-import-container {
    /* [cite: 92] */
    @apply px-4 max-w-full overflow-x-auto; /* [cite: 93] */
  } /* [cite: 93] */

  .file-import-container {
    /* [cite: 93] */
    @apply p-6 max-w-3xl mx-auto; /* [cite: 93] */
  } /* [cite: 94] */

  .excel-inspired-import {
    /* [cite: 94] */
    @apply bg-white rounded-lg shadow-md border border-gray-200; /* [cite: 94] */
  } /* [cite: 95] */

  .import-header {
    /* [cite: 95] */
    @apply p-4 border-b border-gray-200; /* [cite: 95] */
  } /* [cite: 96] */

  .import-header h2 {
    /* [cite: 96] */
    @apply text-xl font-semibold text-gray-800; /* [cite: 96] */
  } /* [cite: 97] */

  .import-subtitle {
    /* [cite: 97] */
    @apply text-sm text-gray-500 mt-1; /* [cite: 97] */
  } /* [cite: 98] */

  .import-dropzone {
    /* [cite: 98] */
    @apply p-8 flex flex-col items-center justify-center cursor-pointer transition-all duration-200 hover:bg-gray-50; /* [cite: 98] */
  } /* [cite: 99] */

  .dropzone-content {
    /* [cite: 99] */
    @apply flex flex-col items-center justify-center gap-3; /* [cite: 99] */
  } /* [cite: 100] */

  .excel-icon {
    /* [cite: 100] */
    @apply mb-3; /* [cite: 100] */
  } /* [cite: 100] */

  .dropzone-text {
    /* [cite: 100] */
    @apply text-gray-600 text-center; /* [cite: 101] */
  } /* [cite: 101] */

  .primary-button {
    /* [cite: 101] */
    @apply bg-green-600 hover:bg-green-700 text-white; /* [cite: 101] */
  } /* [cite: 102] */

  .selected-file {
    /* [cite: 102] */
    @apply p-4 border-t border-gray-200 flex justify-between items-center text-sm; /* [cite: 102] */
  } /* [cite: 103] */

  .file-name {
    /* [cite: 103] */
    @apply font-medium text-gray-700; /* [cite: 103] */
  } /* [cite: 103] */

  .file-size {
    /* [cite: 103] */
    @apply text-gray-500; /* [cite: 103] */
  } /* [cite: 104] */

  .debug-info {
    /* [cite: 104] */
    @apply rounded border border-gray-300; /* [cite: 104] */
  } /* [cite: 105] */
  /* Add style for the loading overlay */
  .fixed.inset-0 {
    /* Ensure it covers the whole screen */
  }
</style>
