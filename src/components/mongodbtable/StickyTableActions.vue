<!-- src/components/mongodbtable/StickyTableActions.vue -->
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, onUnmounted } from 'vue';
import { TableCell } from '@/components/ui/table';
import { TrashIcon } from '@radix-icons/vue';
import { Button } from '@/components/ui/button';
import { useToast } from '@/components/ui/toast/use-toast';
import { getApiBaseUrl } from '@/utils/api';
import { Input } from '@/components/ui/input';
import { Dialog, DialogContent } from '@/components/ui/dialog';

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
  targetRef?: HTMLElement | null;
}>();

const emit = defineEmits(['deleted', 'delete-start', 'delete-end', 'sidebarClick']);

const openDeleteDialog = () => {
  emit('delete-start', props.documentId);
  confirmationText.value = '';
  showDialog.value = true;
  setTimeout(() => inputRef.value?.focus(), 100);
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
    const response = await fetch(`${API_BASE}/collections/${props.collectionName}/documents/${props.documentId}`, {
      method: 'DELETE',
    });

    const { success, data, error } = await response.json();

    if (success && data.deleted_count > 0) {
      toast({ title: 'Document deleted', description: 'Document was successfully removed' });
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

const handleKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && showDialog.value && confirmationText.value === confirmationRequired && !isDeleting.value) {
    event.preventDefault();
    confirmDelete();
  }
};

// Scroll logic
const scrollElement = ref<HTMLElement | Window>(window);
const showBoom = ref(false);

const updateScroll = () => {
  let scrollLeft = 0, scrollWidth = 0, clientWidth = 0;

  const el = scrollElement.value;
  if (el instanceof HTMLElement) {
    scrollLeft = el.scrollLeft;
    scrollWidth = el.scrollWidth;
    clientWidth = el.clientWidth;
  } else {
    scrollLeft = window.scrollX;
    scrollWidth = document.documentElement.scrollWidth;
    clientWidth = window.innerWidth;
  }

  const actionStart = scrollWidth - 30;
  showBoom.value = scrollLeft + clientWidth < actionStart;
};


const setupScrollListener = () => {
  const el = scrollElement.value;
  const handler = updateScroll;
  (el instanceof HTMLElement ? el : window).addEventListener('scroll', handler, { passive: true });
  updateScroll();
};

const removeScrollListener = () => {
  const el = scrollElement.value;
  (el instanceof HTMLElement ? el : window).removeEventListener('scroll', updateScroll);
};

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown);
  if (props.targetRef) scrollElement.value = props.targetRef;
  setupScrollListener();
  showBoom.value = false;
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  removeScrollListener();
});
</script>

<template>
    <!-- White-transparent overlay placed first but with lower z-index -->
    <div 
      v-if="showBoom" 
      class="bg-white fixed right-0 top-1/2 transform -translate-y-1/2 bg-transparent boom-sidebar z-10"
    ></div>
    
    <!-- Buttons for row table with higher z-index -->
    <TableCell
      v-if="showBoom"
      class="border-[1px] pb-3 fixed right-0 select-none z-20"
    >
    <Button
      variant="ghost"
      size="sm"
      class="excel-delete-button mt-1"
      @click="openDeleteDialog"
      :disabled="isDeleting"
    >
        <TrashIcon class="h-4 w-4" />
      </Button>
    </TableCell>

    <Dialog :open="showDialog" @update:open="(val) => val === false && closeDialog()">
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
          <Input v-model="confirmationText" placeholder="Type confirmation text" ref="inputRef" />
          <p class="text-xs text-gray-500 mt-2">Press Enter to confirm when text is correct</p>
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
.excel-delete-button {
  position: relative; /* required for z-index to apply */
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

/* Boom sidebar styles */
.boom-sidebar {
  width: 57px;
  box-shadow: 0 0 1px rgba(0, 0, 0, 0.5);
  height: 100%;
}
</style>