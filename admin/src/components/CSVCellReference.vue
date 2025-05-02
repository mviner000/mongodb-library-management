<!-- src/views/CSVCellReference.vue -->
<script setup lang="ts">
  import { ArrowUpToLine, Download } from 'lucide-vue-next'
  import { ref, onMounted } from 'vue'
  import { useRoute } from 'vue-router'
  import { getApiBaseUrl, apiFetch } from '@/utils/api'
  import { useToast } from '@/components/ui/toast/use-toast'

  const route = useRoute()
  const { toast } = useToast()

  // State & logging
  const isLoading = ref(false)
  const lastRequestParams = ref<any>(null)
  const lastResponse = ref<any>(null)
  const lastError = ref<any>(null)

  const props = defineProps<{
    selectedCell: { colIndex: number; rowNumber: number } | null
    selectedRows: Set<string>
    isSidebarOpen: boolean
    previewMode?: boolean
  }>()

  const emit = defineEmits<{
    (e: 'reset-selection'): void
  }>()

  // ─────────────────────────────────────────────────────────────────────────────
  // CSV Import: calls /api/csv-import/:collection
  // ─────────────────────────────────────────────────────────────────────────────
  const handleCSVImport = async () => {
    isLoading.value = true
    const collection = route.params.name as string
    const selectedIds = Array.from(props.selectedRows)

    // Log request
    const requestInfo = {
      action: 'CSVImport',
      collection,
      items: selectedIds,
      timestamp: new Date().toISOString(),
    }
    console.log('📤 CSV Import Request:', requestInfo)
    lastRequestParams.value = requestInfo

    try {
      // POST to CSV-import endpoint
      const result = await apiFetch<{
        inserted_count: number
        modified_count: number
        errors: string[]
      }>(`/api/csv-import/${collection}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          ids: selectedIds,
          collection: [collection],
        }),
      })

      console.log('✅ CSV Import Response:', result)

      toast({
        title: 'Import Successful',
        description: `${result.inserted_count} inserted, ${result.modified_count} updated.`,
        variant: 'success',
      })

      lastResponse.value = {
        status: 'success',
        timestamp: new Date().toISOString(),
        ...result,
      }

      emit('reset-selection')
    } catch (error) {
      console.error('❌ CSV Import Error:', error)
      lastError.value = error
      toast({
        title: 'Import Failed',
        description:
          error instanceof Error ? error.message : 'Failed to process import. Please try again.',
        variant: 'destructive',
      })
    } finally {
      isLoading.value = false
    }
  }

  // ─────────────────────────────────────────────────────────────────────────────
  // CSV Download (unchanged from before)
  // ─────────────────────────────────────────────────────────────────────────────
  const handleDownloadCSV = async (type: 'single' | 'batch') => {
    const collection = route.params.name as string
    const selectedIds = Array.from(props.selectedRows)
    isLoading.value = true

    try {
      const baseUrl = getApiBaseUrl()
      const url = `${baseUrl}/api/csv-temp/${collection}/download-csv`

      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ids: selectedIds }),
      })
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}))
        throw new Error(errorData.message || `HTTP ${response.status}`)
      }

      const blob = await response.blob()
      const downloadUrl = window.URL.createObjectURL(blob)
      const timestamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, '-')
      const filename = `${collection}_${type}_${timestamp}.csv`

      const a = document.createElement('a')
      a.href = downloadUrl
      a.download = filename
      document.body.appendChild(a)
      a.click()
      window.URL.revokeObjectURL(downloadUrl)
      document.body.removeChild(a)

      toast({
        title: '✅ Download Completed',
        description: `CSV file "${filename}" saved.`,
        variant: 'success',
      })

      console.log('✅ CSV Download Completed')
      lastResponse.value = {
        status: 'completed',
        timestamp: new Date().toISOString(),
        items: selectedIds.length,
        filename,
      }
    } catch (error) {
      console.error('❌ CSV Download Error:', error)
      lastError.value = error
      toast({
        title: 'Download Failed',
        description: error instanceof Error ? error.message : 'Failed to download file',
        variant: 'destructive',
      })
    } finally {
      isLoading.value = false
    }
  }

  // Global error boundary and mount log
  onMounted(() => {
    window.addEventListener('error', (event) => {
      console.error('🔥 Global Error in CSV Component:', {
        message: event.message,
        source: event.filename,
        lineNo: event.lineno,
        columnNo: event.colno,
        error: event.error,
      })
      lastError.value = {
        type: 'global',
        message: event.message,
        error: event.error,
      }
      toast({
        title: 'An error occurred',
        description: event.message,
        variant: 'destructive',
      })
      event.preventDefault()
    })

    console.log('📊 CSV Component Mounted', {
      collection: route.params.name,
      previewMode: props.previewMode,
    })
  })
</script>

<template>
  <!-- CSVCellReference main div -->
  <div
    class="fixed h-[42px] z-30 top-14 w-screen flex items-center bg-white border-b border-b-gray-400 transition-all duration-300 ease-in-out"
    :class="[isSidebarOpen ? 'left-[280px]' : 'left-0', { hidden: previewMode }]"
  >
    <!-- Cell reference box (e.g., CSV) -->
    <div class="flex items-center px-2">
      <div class="flex items-center cursor-pointer">
        <span class="text-sm font-bold text-gray-700">CSV</span>
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
      <!-- single selection button -->
      <div
        v-if="selectedRows.size === 1"
        class="mr-5 flex gap-2"
      >
        <!-- Single CSVImport Button -->
        <button
          @click="handleCSVImport"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-green-100 text-green-600 border-green-300 hover:bg-green-200"
          :disabled="isLoading"
        >
          <ArrowUpToLine class="h-3 w-3 mr-1" />
          <span v-if="isLoading">Processing...</span>
          <span v-else>CSVImport 1 Item</span>
        </button>

        <!-- Single Download Button -->
        <button
          @click="() => handleDownloadCSV('single')"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-blue-100 text-blue-600 border-blue-300 hover:bg-blue-200"
          :disabled="isLoading"
        >
          <Download class="h-3 w-3 mr-1" />
          <span v-if="isLoading">Processing...</span>
          <span v-else>Download Valid table CSV file</span>
        </button>
      </div>

      <!-- buttons for multiple selections -->
      <div
        v-if="selectedRows.size > 1"
        class="mr-5 flex gap-2"
      >
        <!-- Batch CSVImport Button -->
        <button
          @click="handleCSVImport"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-green-100 text-green-600 border-green-300 hover:bg-green-200"
          :disabled="isLoading"
        >
          <ArrowUpToLine class="h-3 w-3 mr-1" />
          <span v-if="isLoading">Processing...</span>
          <span v-else>Batch CSVImport {{ selectedRows.size }} Items</span>
        </button>

        <!-- Batch Download Button -->
        <button
          @click="() => handleDownloadCSV('batch')"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-blue-100 text-blue-600 border-blue-300 hover:bg-blue-200"
          :disabled="isLoading"
        >
          <Download class="h-3 w-3 mr-1" />
          <span v-if="isLoading">Processing...</span>
          <span v-else>Download Valid table CSV file {{ selectedRows.size }} Items</span>
        </button>
      </div>
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
</style>
