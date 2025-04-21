<!-- src/views/CSVImportView.vue -->
<script setup lang="ts">
  import { ref, computed, onMounted, onUnmounted } from 'vue'
  import { useRoute } from 'vue-router'
  import { useToast } from '@/components/ui/toast'
  import { Button } from '@/components/ui/button'
  import MongoDBDataTable from '@/components/MongoDBDataTable.vue'
  import { parseCSV } from '@/utils/parseCSV'
  import { useDataTableStore } from '@/store/dataTableStore'

  const route = useRoute()
  const { toast } = useToast()
  const dataTableStore = useDataTableStore()
  const collectionName = computed(() => route.params.name as string)
  const collectionSchema = computed(() => dataTableStore.collectionSchema)
  const previewData = ref<any[]>([])
  const fileInput = ref<HTMLInputElement | null>(null)
  const hasImportedData = ref(false)

  const handleFileUpload = async (event: Event) => {
    const file = (event.target as HTMLInputElement).files?.[0]
    if (!file) return

    try {
      const { data: csvData } = await parseCSV(file)
      const schema = collectionSchema.value.properties || {}

      // Validate CSV data
      const errors = validateCSVData(csvData, schema)
      if (errors.length > 0) {
        const errorList = errors
          .slice(0, 3)
          .map((e) => `Row ${e.row}: ${e.message}`)
          .join('\n')
        toast({
          title: `Validation Failed (${errors.length} errors)`,
          description: errorList + (errors.length > 3 ? '\n...' : ''),
          variant: 'destructive',
          duration: 10000, // Show longer for multiple errors
        })
        return // Don't proceed if errors exist
      }

      // Proceed with transformation if validation passes
      const shortNameMap = createShortNameMap(schema)
      const transformedData = transformCSVData(csvData, shortNameMap)

      // After transforming data
      previewData.value = transformedData
      dataTableStore.selectedRows = new Set(transformedData.map((doc: any) => doc._id.$oid))

      localStorage.setItem(
        `csv-import-${collectionName.value}`,
        JSON.stringify({
          collection: collectionName.value,
          data: transformedData,
        })
      )

      hasImportedData.value = true
      toast({ title: 'CSV Validated', description: 'Data successfully imported' })
    } catch (error: any) {
      // Handle parsing/validation errors
      toast({ title: 'Import Failed', description: error.message, variant: 'destructive' })
    }
  }

  // Helper functions
  const validateCSVData = (
    data: any[],
    schema: Record<string, any>
  ): Array<{ row: number; message: string }> => {
    const errors: Array<{ row: number; message: string }> = []
    const requiredFields = collectionSchema.value.required || []

    data.forEach((row, index) => {
      // Check required fields
      requiredFields.forEach((field: string) => {
        if (!(field in row) || row[field] === '') {
          errors.push({ row: index + 1, message: `Missing required field: ${field}` })
        }
      })

      // Type validation
      Object.entries(row).forEach(([field, value]) => {
        const fieldSchema = schema[field]
        if (!fieldSchema) return

        const type = Array.isArray(fieldSchema.bsonType)
          ? fieldSchema.bsonType[0]
          : fieldSchema.bsonType

        if (type === 'number' && isNaN(Number(value))) {
          errors.push({ row: index + 1, message: `Invalid number in ${field}` })
        }
        // Add more type checks as needed
      })
    })

    return errors
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

  const transformCSVData = (data: any[], shortNameMap: Record<string, string>) => {
    return data.map((row) => {
      // Generate a new MongoDB-style ObjectId for each row
      const newObjectId = generateMongoObjectId()

      const transformed = Object.entries(row).reduce(
        (acc, [key, value]) => {
          // Skip _id or id columns from the CSV
          if (key === '_id' || key === 'id') {
            return acc
          }

          const fieldName = Object.keys(shortNameMap).find((k) => shortNameMap[k] === key) || key
          acc[fieldName] = value
          return acc
        },
        {} as Record<string, any>
      )

      // Add the generated ObjectId
      transformed._id = { $oid: newObjectId }

      // Add default fields if missing
      if (!transformed.created_at) transformed.created_at = new Date().toISOString()
      if (!transformed.updated_at) transformed.updated_at = new Date().toISOString()

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

  onMounted(() => {
    const savedData = localStorage.getItem(`csv-import-${collectionName.value}`)
    if (savedData) {
      try {
        const csvData = JSON.parse(savedData)
        previewData.value = csvData.data
        hasImportedData.value = true

        // Auto-select all rows when loading saved data
        const ids = csvData.data.map((doc: any) => doc._id.$oid)
        dataTableStore.selectedRows = new Set(ids)

        console.log(`Loaded saved CSV data with ${csvData.data.length} rows`)
      } catch (error) {
        console.error('Error loading saved CSV data:', error)
      }
    }
  })

  const triggerFileSelect = () => {
    fileInput.value?.click()
  }

  const resetImport = () => {
    localStorage.removeItem(`csv-import-${collectionName.value}`)
    previewData.value = []
    hasImportedData.value = false
    // Clear selected rows when resetting
    dataTableStore.selectedRows = new Set()
    if (fileInput.value) {
      fileInput.value.value = ''
    }
    toast({ title: 'Reset Complete', description: 'CSV data has been cleared' })
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
