<script setup lang="ts">
  import { ref } from 'vue'
  import { TableCell } from '@/components/ui/table'
  import { TrashIcon } from '@radix-icons/vue'
  import { Button } from '@/components/ui/button'
  import DeleteDocumentAction from './DeleteDocumentAction.vue'

  const props = defineProps<{
    collectionName: string
    documentId: string
    rowNumber: number
    previewMode: Boolean
  }>()

  const emit = defineEmits(['deleted', 'delete-start', 'delete-end'])

  const deleteActionRef = ref<InstanceType<typeof DeleteDocumentAction> | null>(null)
  const isDialogOpen = ref(false)

  const triggerDelete = () => {
    deleteActionRef.value?.openDeleteDialog()
  }

  // Handle events to track dialog state
  const handleDeleteStart = (id: string) => {
    isDialogOpen.value = true
    emit('delete-start', id)
  }

  const handleDeleteEnd = () => {
    isDialogOpen.value = false
    emit('delete-end')
  }
</script>

<template>
  <TableCell
    class="excel-delete-cell select-none z-0"
    :class="{ 'with-green-border': isDialogOpen }"
  >
    <Button
      variant="ghost"
      size="sm"
      class="excel-delete-button"
      @click="triggerDelete"
      :disabled="previewMode"
    >
      <TrashIcon class="h-4 w-4" />
    </Button>

    <DeleteDocumentAction
      ref="deleteActionRef"
      :collection-name="props.collectionName"
      :document-id="props.documentId"
      :row-number="props.rowNumber"
      @deleted="emit('deleted')"
      @delete-start="handleDeleteStart"
      @delete-end="handleDeleteEnd"
    />
  </TableCell>
</template>

<style scoped>
  .excel-delete-cell {
    /* Applied to TD */
    border: 1px solid #d0d0d0;
    position: sticky;
    right: 0;
    width: 60px !important;
    min-width: 60px !important;
    max-width: 60px !important;
    background-color: #ffffff;
    text-align: center;
    vertical-align: middle;
    box-sizing: border-box;
  }

  /* Add the green border only when the class is applied */
  .with-green-border {
    border-right: 2px solid #ef4444;
  }

  .excel-delete-button {
    color: #d32f2f;
  }
  .excel-delete-button:hover {
    color: #b71c1c;
    background-color: #ffebee;
  }
</style>
