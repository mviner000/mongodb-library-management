<!-- src/views/CSVImportView.vue -->
<script setup lang="ts">
  import { ref, computed, onMounted } from 'vue'
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

      previewData.value = transformedData
      localStorage.setItem(
        `csv-import-${collectionName.value}`,
        JSON.stringify({
          collection: collectionName.value,
          data: transformedData,
        })
      )

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
        console.log(`Loaded saved CSV data with ${csvData.data.length} rows`)
      } catch (error) {
        console.error('Error loading saved CSV data:', error)
      }
    }
  })

  const triggerFileSelect = () => {
    fileInput.value?.click()
  }
</script>

<template>
  <div class="csv-import-container">
    <input
      ref="fileInput"
      type="file"
      accept=".csv"
      class="hidden"
      @change="handleFileUpload"
    />

    <div class="import-controls mb-4">
      <Button @click="triggerFileSelect">Import CSV</Button>
      <span class="ml-2 text-sm text-gray-600">
        Selected file: {{ fileInput?.files?.[0]?.name || 'None' }}
      </span>
    </div>

    <MongoDBDataTable
      :selected-collection="collectionName"
      :preview-data="previewData"
      preview-mode
    />
  </div>
</template>

<style scoped>
  .csv-import-container {
    @apply p-4 max-w-full overflow-x-auto;
  }

  .import-controls {
    @apply sticky left-0 bg-white z-50 p-2 border-b;
  }
</style>
