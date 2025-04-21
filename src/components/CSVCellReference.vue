<!-- src/views/ExcelCellReference.vue -->
<script setup lang="ts">
  import { ArrowUpToLine } from 'lucide-vue-next'

  defineProps<{
    selectedCell: { colIndex: number; rowNumber: number } | null
    selectedRows: Set<string>
    isSidebarOpen: boolean
    previewMode?: boolean
  }>()

  const emit = defineEmits<{
    (e: 'reset-selection'): void
  }>()

  // Placeholder function for recovery
  const handleCSVImport = () => {
    emit('reset-selection')
  }
</script>

<template>
  <!-- ExcelCellReference main div -->
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
        >
          <ArrowUpToLine class="h-3 w-3 mr-1" />
          CSVImport 1 Item
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
        >
          <ArrowUpToLine class="h-3 w-3 mr-1" />
          Batch CSVImport {{ selectedRows.size }} Items
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
