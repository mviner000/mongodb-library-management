<!-- src/views/CSVImportView.vue -->
<script setup lang="ts">
  import { ref, computed, onMounted, onUnmounted } from 'vue'
  import { useRoute } from 'vue-router'
  import { useToast } from '@/components/ui/toast'
  import { Button } from '@/components/ui/button'
  import { storeToRefs } from 'pinia'
  import MongoDBDataTable from '@/components/MongoDBDataTable.vue'
  import { parseCSV } from '@/utils/parseCSV'
  import { useDataTableStore } from '@/store/dataTableStore'
  import { getApiBaseUrl } from '@/utils/api'

  const route = useRoute()
  const { toast } = useToast()
  const dataTableStore = useDataTableStore()
  const { collectionSchema } = storeToRefs(dataTableStore)
  const collectionName = computed(() => route.params.name as string)
  const primaryKey = computed(() => {
    return collectionSchema.value?.primaryKey
  })
  const previewData = ref<any[]>([])
  const fileInput = ref<HTMLInputElement | null>(null)
  const hasImportedData = ref(false)

  const handleFileUpload = async (event: Event) => {
    const file = (event.target as HTMLInputElement).files?.[0]
    if (!file) return

    try {
      const { data: csvData } = await parseCSV(file)
      const schema = collectionSchema.value.properties || {}

      // Create short name map for field mapping
      const shortNameMap = createShortNameMap(schema)

      // Check for required columns
      const csvHeaders = csvData.length > 0 ? Object.keys(csvData[0]) : []
      const missingRequired = primaryKey.value
        ? !csvHeaders.includes(shortNameMap[primaryKey.value] || primaryKey.value)
          ? [primaryKey.value]
          : []
        : []

      if (missingRequired.length > 0) {
        toast({
          title: 'Invalid CSV Format',
          description: `Missing primary key column: ${missingRequired.join(', ')}`,
          variant: 'destructive',
        })
        return
      }

      // Validate CSV data and split into valid/invalid
      const { valid: validData, invalid: invalidData } = validateCSVData(csvData, schema)

      // Show upload counts in toast
      const validCount = validData.length
      const invalidCount = invalidData.length
      toast({
        title: `CSV Import Results`,
        description: `Valid data: ${validCount}, Invalid data: ${invalidCount}`,
        duration: 5000,
      })

      // Transform valid data for storage
      const transformedValid = transformCSVData(validData, shortNameMap, collectionSchema.value)

      // UPDATED: Transform invalid data to include all CSV columns
      const transformedInvalid = invalidData.map(({ row, errors }) => {
        const transformedRow: Record<string, string> = {}

        // Use CSV headers to include all columns from the original CSV
        csvHeaders.forEach((header) => {
          const value = row[header]
          transformedRow[header] = value !== undefined ? String(value) : ''
        })

        return {
          ...transformedRow,
          errors,
        }
      })

      // Send to backend
      const response = await fetch(`${getApiBaseUrl()}/api/csv-temp/${collectionName.value}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ valid: transformedValid, invalid: transformedInvalid }),
      })

      if (!response.ok) throw new Error('Failed to save temporary data')

      // After transforming data
      previewData.value = transformedValid
      // Use MongoDB-compatible ObjectID for selection
      dataTableStore.selectedRows = new Set(transformedValid.map((doc: any) => doc._id.$oid))
      hasImportedData.value = true
      toast({ title: 'CSV Processed', description: 'Data successfully imported' })
    } catch (error: any) {
      // Handle parsing/validation errors
      toast({ title: 'Import Failed', description: error.message, variant: 'destructive' })
    }
  }

  // Helper functions
  const validateCSVData = (
    data: any[],
    schema: Record<string, any>
  ): { valid: any[]; invalid: Array<{ row: any; errors: string[] }> } => {
    const valid: any[] = []
    const invalid: Array<{ row: any; errors: string[] }> = []

    data.forEach((row, index) => {
      const errors: string[] = []

      // Primary key field check
      if (primaryKey.value) {
        const value = row[primaryKey.value]
        if (value === null || value === undefined || value === '') {
          errors.push(`Missing primary key field: ${primaryKey.value}`)
        }
      }

      // Type validation
      Object.entries(row).forEach(([field, value]) => {
        const fieldSchema = schema[field]
        if (!fieldSchema || value === null || value === '') return

        const type = Array.isArray(fieldSchema.bsonType)
          ? fieldSchema.bsonType[0]
          : fieldSchema.bsonType

        if (
          (type === 'number' || type === 'int' || type === 'double' || type === 'long') &&
          isNaN(Number(value))
        ) {
          errors.push(`Invalid number in ${field}`)
        }

        if (
          type === 'bool' &&
          !['true', 'false', '0', '1', 'yes', 'no', ''].includes(String(value).toLowerCase())
        ) {
          errors.push(`Invalid boolean value in ${field}`)
        }

        if (type === 'date' && isNaN(Date.parse(String(value)))) {
          errors.push(`Invalid date in ${field}`)
        }
      })

      if (errors.length > 0) {
        invalid.push({ row, errors })
      } else {
        valid.push(row)
      }
    })

    return { valid, invalid }
  }

  const createShortNameMap = (schema: Record<string, any>) => {
    return Object.keys(schema).reduce(
      (acc, field) => {
        acc[field] = schema[field]?.ui?.short_name || field
        return acc
      },
      {} as Record<string, string>
    )
  }

  const convertValueBySchema = (value: any, fieldSchema: any) => {
    if (value === null || value === undefined || value === '') return null

    const type = Array.isArray(fieldSchema.bsonType)
      ? fieldSchema.bsonType[0]
      : fieldSchema.bsonType

    switch (type) {
      case 'bool':
        if (value === '1' || value === 'yes' || String(value).toLowerCase() === 'true') return true
        if (value === '0' || value === 'no' || String(value).toLowerCase() === 'false') return false
        return null
      case 'int':
      case 'long':
        return parseInt(value, 10)
      case 'double':
      case 'number':
        return parseFloat(value)
      case 'date':
        try {
          return new Date(value).toISOString()
        } catch {
          return null
        }
      default:
        return value
    }
  }

  const transformCSVData = (data: any[], shortNameMap: Record<string, string>, schema: any) => {
    const schemaFields = Object.keys(schema.properties || {})

    return data.map((row) => {
      const transformed: Record<string, any> = {}

      // Map all schema fields
      schemaFields.forEach((field) => {
        // Find the CSV column that corresponds to this schema field
        const csvKey =
          Object.entries(shortNameMap).find(([_, shortName]) => shortName === field)?.[0] || field

        const rawValue = row[csvKey]
        // Convert values based on schema type
        transformed[field] = schema.properties[field]
          ? convertValueBySchema(rawValue, schema.properties[field])
          : rawValue === '' || rawValue === undefined
            ? null
            : rawValue
      })

      // Add MongoDB-compatible ObjectID instead of numeric ID
      transformed._id = { $oid: generateMongoObjectId() }

      // Add timestamps if they're part of the schema
      const now = new Date().toISOString()
      if (schemaFields.includes('created_at')) {
        transformed.created_at = transformed.created_at || now
      }

      if (schemaFields.includes('updated_at')) {
        transformed.updated_at = transformed.updated_at || now
      }

      return transformed
    })
  }

  // Function to generate MongoDB-compatible ObjectIDs
  function generateMongoObjectId(): string {
    const timestamp = Math.floor(new Date().getTime() / 1000)
      .toString(16)
      .padStart(8, '0')
    const machineId = randomHexString(6)
    const processId = randomHexString(4)
    const counter = randomHexString(6)

    return timestamp + machineId + processId + counter
  }

  // Helper function to generate random hex strings of specified length
  function randomHexString(length: number): string {
    let result = ''
    const characters = '0123456789abcdef'
    for (let i = 0; i < length; i++) {
      result += characters.charAt(Math.floor(Math.random() * characters.length))
    }
    return result
  }

  onMounted(async () => {
    try {
      // Load from SQLite backend
      const response = await fetch(`${getApiBaseUrl()}/api/csv-temp/${collectionName.value}`)

      if (response.ok) {
        const data = await response.json()
        previewData.value = data.valid || data // Support both old and new format
        hasImportedData.value = previewData.value.length > 0

        // Auto-select all rows when loading saved data - use MongoDB ObjectID format
        const ids = previewData.value.map((doc: any) => doc._id?.$oid)
        dataTableStore.selectedRows = new Set(ids.filter(Boolean)) // Filter out any undefined IDs

        console.log(`Loaded saved CSV data with ${previewData.value.length} rows`)
      }
    } catch (error) {
      console.error('Error loading saved CSV data:', error)
    }
  })

  const triggerFileSelect = () => {
    fileInput.value?.click()
  }

  const resetImport = async () => {
    try {
      // Delete from SQLite backend instead of localStorage
      await fetch(`${getApiBaseUrl()}/api/csv-temp/${collectionName.value}`, { method: 'DELETE' })

      previewData.value = []
      hasImportedData.value = false
      // Clear selected rows when resetting
      dataTableStore.selectedRows = new Set()
      if (fileInput.value) {
        fileInput.value.value = ''
      }
      toast({ title: 'Reset Complete', description: 'Temporary data cleared' })
    } catch (error: any) {
      toast({ title: 'Reset Failed', description: error.message, variant: 'destructive' })
    }
  }

  onUnmounted(() => {
    // Clear selected rows when component is unmounted
    dataTableStore.selectedRows = new Set()
  })
</script>

<template>
  <!-- Conditional rendering based on whether data is imported -->
  <div
    v-if="!hasImportedData"
    class="file-import-container"
  >
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
          <div class="mt-4">
            <Button class="primary-button">Browse Files</Button>
          </div>
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

  <!-- Table view when data is imported -->
  <div v-else>
    <div class="csv-import-container">
      <MongoDBDataTable
        :selected-collection="collectionName"
        :preview-data="previewData"
        preview-mode
      />
    </div>
  </div>

  <!-- Reset button - completely separate from the content flow -->
  <button
    v-if="hasImportedData"
    @click="resetImport"
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
    @apply px-4 max-w-full overflow-x-auto;
  }

  .file-import-container {
    @apply p-6 max-w-3xl mx-auto;
  }

  .excel-inspired-import {
    @apply bg-white rounded-lg shadow-md border border-gray-200;
  }

  .import-header {
    @apply p-4 border-b border-gray-200;
  }

  .import-header h2 {
    @apply text-xl font-semibold text-gray-800;
  }

  .import-subtitle {
    @apply text-sm text-gray-500 mt-1;
  }

  .import-dropzone {
    @apply p-8 flex flex-col items-center justify-center cursor-pointer transition-all duration-200 hover:bg-gray-50;
  }

  .dropzone-content {
    @apply flex flex-col items-center justify-center gap-3;
  }

  .excel-icon {
    @apply mb-3;
  }

  .dropzone-text {
    @apply text-gray-600 text-center;
  }

  .primary-button {
    @apply bg-green-600 hover:bg-green-700 text-white;
  }

  .selected-file {
    @apply p-4 border-t border-gray-200 flex justify-between items-center text-sm;
  }

  .file-name {
    @apply font-medium text-gray-700;
  }

  .file-size {
    @apply text-gray-500;
  }
</style>
