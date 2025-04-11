<!-- src/components/ExcelCellReference.vue -->
<template>
  <div class="fixed h-[42px] top-14 left-0 z-40 w-full flex items-center bg-white border-b border-b-gray-400">
    <!-- Cell reference box (e.g., A1) -->
    <div class="flex items-center px-2">
      <div class="flex items-center cursor-pointer">
        <span class="text-sm font-bold text-gray-700">{{ cellReference }}</span>
      </div>
    </div>

    <!-- fx formula indicator -->
    <div class="flex items-center px-3 text-gray-500">
      <span class="text-sm italic">fx</span>
    </div>

    <!-- Empty space -->
    <div class="flex-1 h-full"></div>
    
    <!-- Delete button for single selection -->
    <div v-if="selectedRows.size === 1" class="mr-4">
      <button 
        @click="handleDeleteClick"
        class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-red-100 text-red-500 border-red-300 hover:bg-red-200"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1">
          <path d="M3 6h18"></path>
          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2v2"></path>
        </svg>
        Delete 1 Item
      </button>
    </div>

    <!-- Batch Delete button for multiple selections -->
    <div v-if="selectedRows.size > 1" class="mr-4">
      <button 
        class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-red-100 text-red-500 border-red-300 hover:bg-red-200"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1">
          <path d="M3 6h18"></path>
          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2v2"></path>
        </svg>
        Batch Delete {{ selectedRows.size }} Items
      </button>
    </div>
    
    <!-- Delete document dialog -->
    <DeleteDocumentAction
      v-if="documentToDelete"
      :collection-name="collectionName"
      :document-id="documentToDelete.id"
      :row-number="documentToDelete.rowNumber"
      ref="deleteDocumentRef"
      @deleted="onDocumentDeleted"
      @delete-start="(id) => $emit('delete-start', id)"
      @delete-end="$emit('delete-end')"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import DeleteDocumentAction from './mongodbtable/DeleteDocumentAction.vue';

const props = defineProps<{
  selectedCell: { colIndex: number; rowNumber: number } | null;
  selectedRows: Set<string>;
  collectionName: string;
  documents: any[];
  currentPage: number;
  pageSize: number;
}>();

const emit = defineEmits<{
  (e: 'document-deleted'): void;
  (e: 'delete-start', id: string): void;
  (e: 'delete-end'): void;
}>();

// Reference to the DeleteDocumentAction component
const deleteDocumentRef = ref<InstanceType<typeof DeleteDocumentAction> | null>(null);

// Store information about document to delete
const documentToDelete = ref<{ id: string; rowNumber: number } | null>(null);
const isDeleting = ref(false);

// Debug watcher for props
watch(() => props.selectedRows, (newVal) => {
  console.log('Selected rows updated:', [...newVal]);
}, { immediate: true });

watch(() => props.collectionName, (newVal) => {
  console.log('Collection name:', newVal);
}, { immediate: true });

const getColumnLabel = (index: number): string => {
  let label = '';
  let i = index;
  do {
    const remainder = i % 26;
    label = String.fromCharCode(65 + remainder) + label;
    i = Math.floor(i / 26) - 1;
  } while (i >= 0);
  return label;
};

const cellReference = computed(() => {
  if (!props.selectedCell) return 'A1'; // Default when no cell is selected
  const { colIndex, rowNumber } = props.selectedCell;
  return `${getColumnLabel(colIndex)}${rowNumber}`;
});

// Get the document ID from the selected row
const getSelectedDocumentId = (): string | null => {
  if (props.selectedRows.size !== 1) return null;
  
  // Get the first (and only) item from the Set
  const [documentId] = [...props.selectedRows];
  return documentId;
};

// Calculate the row number based on the selected document ID
const getRowNumberForDocument = (documentId: string): number => {
  // Find the index of the document in the props.documents array
  const index = props.documents.findIndex(doc => doc._id.$oid === documentId);
  
  console.log('Document lookup for ID:', documentId);
  console.log('Document index in array:', index);
  
  if (index === -1) return 0;
  
  // Calculate row number based on pagination
  const rowNumber = (props.currentPage - 1) * props.pageSize + index + 1;
  console.log('Calculated row number:', rowNumber);
  
  return rowNumber;
};

// Handle click on delete button
const handleDeleteClick = () => {
  console.log('Delete button clicked!');
  console.log('Current selected rows:', [...props.selectedRows]);
  
  const documentId = getSelectedDocumentId();
  console.log('Selected document ID:', documentId);
  
  if (!documentId) {
    console.error('No document ID found!');
    return;
  }
  
  const rowNumber = getRowNumberForDocument(documentId);
  console.log('Preparing to delete:');
  console.log('- Collection:', props.collectionName);
  console.log('- Document ID:', documentId);
  console.log('- Row Number:', rowNumber);
  
  documentToDelete.value = { id: documentId, rowNumber };
  console.log('documentToDelete set to:', documentToDelete.value);
  
  // Use a small delay to ensure component is updated
  setTimeout(() => {
    console.log('Opening delete dialog with ref:', deleteDocumentRef.value);
    deleteDocumentRef.value?.openDeleteDialog();
  }, 50);
};

// Event handlers for delete operations
const onDocumentDeleted = () => {
  console.log('Document successfully deleted!');
  documentToDelete.value = null;
  isDeleting.value = false;
  emit('document-deleted');
};

// Debug log when the component mounts
console.log('ExcelCellReference component props:', {
  collectionName: props.collectionName,
  selectedRows: [...props.selectedRows],
  documentsCount: props.documents?.length,
  currentPage: props.currentPage,
  pageSize: props.pageSize
});
</script>