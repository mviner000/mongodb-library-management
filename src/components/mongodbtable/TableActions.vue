<script setup lang="ts">
import { ref } from 'vue'; // Keep ref
// REMOVE: onMounted, onBeforeUnmount
import { TableCell } from '@/components/ui/table';
import { TrashIcon } from '@radix-icons/vue';
import { Button } from '@/components/ui/button';
// REMOVE: useToast, getApiBaseUrl, Input, Dialog, DialogContent
import DeleteDocumentAction from './DeleteDocumentAction.vue'; // IMPORT the new component

// REMOVE: toast, API_BASE, isDeleting, showDialog, confirmationText, confirmationRequired, inputRef

const props = defineProps<{
  collectionName: string;
  documentId: string;
  rowNumber: number;
}>();

// Keep emits, they will be forwarded from the child
const emit = defineEmits(['deleted', 'delete-start', 'delete-end']);

// Add a ref to access the child component's exposed methods
const deleteActionRef = ref<InstanceType<typeof DeleteDocumentAction> | null>(null);

// Method to trigger the delete dialog in the child component
const triggerDelete = () => {
  deleteActionRef.value?.openDeleteDialog();
};

// REMOVE: openDeleteDialog, closeDialog, confirmDelete, handleKeyDown
// REMOVE: onMounted, onBeforeUnmount (as they were only for the keydown listener)

</script>

<template>
  <TableCell class="excel-cell excel-actions-cell select-none">
    <Button
      variant="ghost"
      size="sm"
      class="excel-delete-button"
      @click="triggerDelete" 
      :disabled="false" 
    >
      <TrashIcon class="h-4 w-4" />
    </Button>

    <DeleteDocumentAction
      ref="deleteActionRef"
      :collection-name="props.collectionName"
      :document-id="props.documentId"
      :row-number="props.rowNumber"
      @deleted="emit('deleted')"
      @delete-start="(id) => emit('delete-start', id)"
      @delete-end="emit('delete-end')"
    />

    </TableCell>
</template>

<style scoped>
/* Keep styles for the button */
.excel-delete-button {
  color: #d32f2f;
}
.excel-delete-button:hover {
  color: #b71c1c;
  background-color: #ffebee;
}

/* REMOVE .custom-delete-dialog style */
</style>