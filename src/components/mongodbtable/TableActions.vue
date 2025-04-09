<!-- src/components/mongodbtable/TableActions.vue -->
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { TableCell } from '@/components/ui/table';
import { TrashIcon } from '@radix-icons/vue';
import { Button } from '@/components/ui/button';
import { useToast } from '@/components/ui/toast/use-toast';
import { getApiBaseUrl } from '@/utils/api';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
} from '@/components/ui/dialog';

const { toast } = useToast();
const API_BASE = getApiBaseUrl();
const isDeleting = ref(false);
const showDialog = ref(false);
const confirmationText = ref('');
const confirmationRequired = 'confirm-delete';
const inputRef = ref<HTMLInputElement | null>(null);

const props = defineProps<{
  collectionName: string;
  documentId: string;
  rowNumber: number;
}>();

const emit = defineEmits(['deleted', 'delete-start', 'delete-end']);

const openDeleteDialog = () => {
  emit('delete-start', props.documentId);
  confirmationText.value = '';
  showDialog.value = true;
  // Focus the input after dialog is open
  setTimeout(() => {
    if (inputRef.value) inputRef.value.focus();
  }, 100);
};

const closeDialog = () => {
  showDialog.value = false;
  confirmationText.value = '';
  emit('delete-end');
};

const confirmDelete = async () => {
  if (confirmationText.value !== confirmationRequired) return;
  
  try {
    isDeleting.value = true;
    const response = await fetch(
      `${API_BASE}/collections/${props.collectionName}/documents/${props.documentId}`,
      { method: 'DELETE' }
    );

    const { success, data, error } = await response.json();
    
    if (success && data.deleted_count > 0) {
      toast({
        title: 'Document deleted',
        description: 'Document was successfully removed',
      });
      emit('deleted');
    } else {
      toast({
        title: 'Delete failed',
        description: error || 'Failed to delete document',
        variant: 'destructive',
      });
    }
  } catch (error) {
    toast({
      title: 'Error deleting document',
      description: String(error),
      variant: 'destructive',
    });
  } finally {
    isDeleting.value = false;
    showDialog.value = false;
    emit('delete-end');
  }
};

// Handle Enter key press within the dialog
const handleKeyDown = (event: KeyboardEvent) => {
  if (
    event.key === 'Enter' && 
    showDialog.value && 
    confirmationText.value === confirmationRequired &&
    !isDeleting.value
  ) {
    event.preventDefault();
    confirmDelete();
  }
};

// Set up and clean up event listeners
onMounted(() => {
  document.addEventListener('keydown', handleKeyDown);
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <TableCell class="excel-cell excel-actions-cell select-none">
    <Button
      variant="ghost"
      size="sm"
      class="excel-delete-button"
      @click="openDeleteDialog"
      :disabled="isDeleting"
    >
      <TrashIcon class="h-4 w-4" />
    </Button>
    
    <Dialog 
      :open="showDialog" 
      @update:open="(val) => val === false && closeDialog()"
    >
      <DialogContent 
        class="custom-delete-dialog p-0 overflow-hidden border-rose-200"
        @keydown.enter.prevent="confirmationText === confirmationRequired && !isDeleting && confirmDelete()"
      >
        <div class="bg-rose-100 text-rose-700 p-4 border-b border-rose-200 flex items-center border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground">
          <div class="flex-1">
            You are about to delete row number: {{ props.rowNumber }}
          </div>
        </div>
        
        <div class="p-4 bg-white">
          <p class="text-sm text-gray-700 mb-3">
            To confirm, type <span class="font-medium text-rose-600">{{ confirmationRequired }}</span> in the box below
          </p>
          <Input 
            v-model="confirmationText"
            placeholder="Type confirmation text"
            ref="inputRef"
          />
          <p class="text-xs text-gray-500 mt-2">
            Press Enter to confirm when text is correct
          </p>
        </div>
        
        <div class="flex justify-end p-3 pt-0 bg-white">
          <Button
            @click="confirmDelete"
            size="lg"
            class="w-full bg-rose-600 hover:bg-rose-700 text-white"
            :disabled="confirmationText !== confirmationRequired || isDeleting"
          >
            {{ isDeleting ? 'Deleting...' : 'Delete this document' }}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  </TableCell>
</template>

<style scoped>
/* Inherit styles from parent table */
.excel-delete-button {
  color: #d32f2f;
}

.excel-delete-button:hover {
  color: #b71c1c;
  background-color: #ffebee;
}

.custom-delete-dialog {
  max-width: 500px;
  border-radius: 4px;
}
</style>