<template>
  <div class="fixed h-[42px] z-30 top-14 left-0 w-full flex items-center bg-white border-b border-b-gray-400">
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
        @click="openBatchDeleteDialog"
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
    
    <!-- Single Delete document dialog -->
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

    <!-- Batch Delete confirmation dialog -->
    <Dialog
      :open="showBatchDeleteDialog"
      @update:open="(val) => val === false && closeBatchDeleteDialog()"
    >
      <DialogContent
        class="custom-delete-dialog p-0 overflow-hidden border-rose-200"
        @keydown.enter.prevent="batchConfirmationText === confirmationRequired && !isBatchDeleting && confirmBatchDelete()"
      >
        <DialogTitle class="sr-only">Batch Delete Confirmation</DialogTitle>
        <DialogDescription class="sr-only">
          Please confirm your intention to delete {{ selectedRows.size }} documents by typing the confirmation text.
        </DialogDescription>
        
        <div class="bg-rose-100 text-rose-700 p-4 border-b border-rose-200 flex items-center">
          <div class="flex-1">
            You are about to delete {{ selectedRows.size }} documents
          </div>
        </div>

        <div class="p-4 bg-white">
          <p class="text-sm text-gray-700 mb-3">
            To confirm, type <span class="font-medium text-rose-600">{{ confirmationRequired }}</span> in the box below
          </p>
          <input
            v-model="batchConfirmationText"
            placeholder="Type confirmation text"
            ref="batchInputRef"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rose-500"
            aria-label="Confirmation text"
          />
          <p class="text-xs text-gray-500 mt-2">
            Press Enter to confirm when text is correct
          </p>
        </div>

        <div class="flex justify-end p-3 pt-0 bg-white">
          <Button
            @click="confirmBatchDelete"
            size="lg"
            class="w-full bg-rose-600 hover:bg-rose-700 text-white"
            :disabled="batchConfirmationText !== confirmationRequired || isBatchDeleting"
          >
            {{ isBatchDeleting ? 'Deleting...' : `Delete ${selectedRows.size} documents` }}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue';
import DeleteDocumentAction from './mongodbtable/DeleteDocumentAction.vue';
import { getApiBaseUrl } from '@/utils/api';
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { useToast } from '@/components/ui/toast/use-toast';

const API_BASE = getApiBaseUrl();
const { toast } = useToast();

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
  (e: 'reset-selection'): void;
}>();

// Reference to the DeleteDocumentAction component
const deleteDocumentRef = ref<InstanceType<typeof DeleteDocumentAction> | null>(null);

// Batch delete dialog state
const showBatchDeleteDialog = ref(false);
const isBatchDeleting = ref(false);
const batchConfirmationText = ref('');
const confirmationRequired = 'confirm-delete';
const batchInputRef = ref<HTMLInputElement | null>(null);

// Open batch delete dialog
const openBatchDeleteDialog = () => {
  emit('delete-start', 'batch');
  batchConfirmationText.value = '';
  showBatchDeleteDialog.value = true;
  // Focus the input after dialog is open
  setTimeout(() => {
    batchInputRef.value?.focus();
  }, 100);
};

// Close batch delete dialog
const closeBatchDeleteDialog = () => {
  showBatchDeleteDialog.value = false;
  batchConfirmationText.value = '';
  // Only emit delete-end if the dialog was closed manually,
  // not after a successful/failed delete (handled in confirmBatchDelete)
  if (!isBatchDeleting.value) {
    emit('delete-end');
  }
};

// Add batch delete handler
const confirmBatchDelete = async () => {
  if (batchConfirmationText.value !== confirmationRequired || isBatchDeleting.value) return;
  
  isBatchDeleting.value = true;
  try {
    // Show processing notification
    const response = await fetch(
      `${API_BASE}/collections/${props.collectionName}/documents/batch-delete`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ids: [...props.selectedRows] })
      }
    );

    const { success, error } = await response.json();
    
    if (success) {
      toast({
        title: 'Documents deleted',
        description: `${props.selectedRows.size} documents were successfully removed`,
      });
      emit('document-deleted');
      emit('reset-selection');
      showBatchDeleteDialog.value = false; // Close dialog on success
    } else {
      toast({
        title: 'Batch delete failed',
        description: error || 'Failed to delete documents',
        variant: 'destructive',
      });
    }
  } catch (error) {
    toast({
      title: 'Error deleting documents',
      description: String(error),
      variant: 'destructive',
    });
  } finally {
    isBatchDeleting.value = false;
    // If the dialog is still open (e.g., on failure), keep it open until user closes,
    // but if it was successful, it's already closed above.
    emit('delete-end');
    // Ensure confirmation text is cleared if delete fails and dialog stays open
    if (showBatchDeleteDialog.value) {
      batchConfirmationText.value = '';
    }
  }
};

// Handle Enter key press for batch dialog
const handleBatchKeyDown = (event: KeyboardEvent) => {
  if (
    event.key === 'Enter' &&
    showBatchDeleteDialog.value &&
    batchConfirmationText.value === confirmationRequired &&
    !isBatchDeleting.value
  ) {
    event.preventDefault();
    confirmBatchDelete();
  }
};

// Set up and clean up global event listener for Enter key
onMounted(() => {
  document.addEventListener('keydown', handleBatchKeyDown);
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleBatchKeyDown);
});

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
  emit('reset-selection'); // Also reset selection after single document deletion
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

<style scoped>
.custom-delete-dialog {
  max-width: 500px;
  border-radius: 4px;
}

/* Keep button styles within the dialog consistent */
.bg-rose-100 { background-color: #ffebee; }
.text-rose-700 { color: #c6282d; }
.border-rose-200 { border-color: #ffcdd2; }
.text-rose-600 { color: #e53935; }
.bg-rose-600 { background-color: #e53935; }
.hover\:bg-rose-700:hover { background-color: #d32f2f; }
</style>