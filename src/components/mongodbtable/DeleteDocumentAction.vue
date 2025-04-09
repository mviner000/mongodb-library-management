<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
} from '@/components/ui/dialog';
import { useToast } from '@/components/ui/toast/use-toast';
import { getApiBaseUrl } from '@/utils/api';

// --- Props ---
const props = defineProps<{
  collectionName: string;
  documentId: string;
  rowNumber: number;
}>();

// --- Emits ---
const emit = defineEmits<{
  (e: 'deleted'): void;
  (e: 'delete-start', documentId: string): void;
  (e: 'delete-end'): void;
}>();

// --- State ---
const { toast } = useToast();
const API_BASE = getApiBaseUrl();
const isDeleting = ref(false);
const showDialog = ref(false);
const confirmationText = ref('');
const confirmationRequired = 'confirm-delete';
const inputRef = ref<HTMLInputElement | null>(null);

// --- Methods ---
const openDeleteDialog = () => {
  emit('delete-start', props.documentId);
  confirmationText.value = '';
  showDialog.value = true;
  // Focus the input after dialog is open and rendered
  setTimeout(() => {
    inputRef.value?.focus();
  }, 100); // Small delay might be needed for DOM update
};

const closeDialog = () => {
  showDialog.value = false;
  confirmationText.value = '';
  // Only emit delete-end if the dialog was closed manually,
  // not after a successful/failed delete (handled in confirmDelete)
  if (!isDeleting.value) {
      emit('delete-end');
  }
};

const confirmDelete = async () => {
  if (confirmationText.value !== confirmationRequired || isDeleting.value) return;

  isDeleting.value = true; // Set deleting state immediately
  try {
    const response = await fetch(
      `${API_BASE}/collections/${props.collectionName}/documents/${props.documentId}`,
      { method: 'DELETE' }
    );

    const { success, data, error } = await response.json();

    if (success && data?.deleted_count > 0) {
      toast({
        title: 'Document deleted',
        description: 'Document was successfully removed',
      });
      emit('deleted'); // Emit deleted event on success
      showDialog.value = false; // Close dialog on success
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
    // Whether success or failure, reset deleting state and emit end event
    isDeleting.value = false;
    // If the dialog is still open (e.g., on failure), keep it open until user closes,
    // but if it was successful, it's already closed above.
    // Emit delete-end *after* state is reset.
    emit('delete-end');
    // Ensure confirmation text is cleared if delete fails and dialog stays open
     if (showDialog.value) {
        confirmationText.value = '';
     }
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
    event.preventDefault(); // Prevent potential form submission if wrapped
    confirmDelete();
  }
};

// --- Lifecycle Hooks ---
// Set up and clean up global event listener for Enter key specifically for this dialog instance
onMounted(() => {
  document.addEventListener('keydown', handleKeyDown);
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeyDown);
});

// --- Expose ---
// Make the openDeleteDialog method callable from the parent component via a ref
defineExpose({
  openDeleteDialog,
});

</script>

<template>
  <Dialog
    :open="showDialog"
    @update:open="(val) => val === false && closeDialog()"
  >
    <DialogContent
      class="custom-delete-dialog p-0 overflow-hidden border-rose-200"
      @keydown.enter.prevent="confirmationText === confirmationRequired && !isDeleting && confirmDelete()"
    >
      <div class="bg-rose-100 text-rose-700 p-4 border-b border-rose-200 flex items-center">
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
</template>

<style scoped>
.custom-delete-dialog {
  max-width: 500px;
  border-radius: 4px;
  /* Styles previously applied via hover on parent components are moved here if needed */
  /* Example:
  border: 1px solid #ccc;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1); */
}

/* Keep button styles within the dialog consistent */
.bg-rose-100 { background-color: #ffebee; }
.text-rose-700 { color: #c6282d; }
.border-rose-200 { border-color: #ffcdd2; }
.text-rose-600 { color: #e53935; }
.bg-rose-600 { background-color: #e53935; }
.hover\:bg-rose-700:hover { background-color: #d32f2f; }
</style>