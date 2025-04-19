<!-- src/components/mongodbtable/StickyTableActions.vue -->

<script setup lang="ts">
  import { ref, onMounted, onBeforeUnmount, onUnmounted } from 'vue'
  import { TableCell } from '@/components/ui/table'
  import { TrashIcon } from '@radix-icons/vue'
  import { Button } from '@/components/ui/button'
  // REMOVE: useToast, getApiBaseUrl, Input, Dialog, DialogContent
  import DeleteDocumentAction from './DeleteDocumentAction.vue' // IMPORT the new component

  // REMOVE: toast, API_BASE, isDeleting, showDialog, confirmationText, confirmationRequired, inputRef

  const props = defineProps<{
    collectionName: string
    documentId: string
    rowNumber: number
    targetRef?: HTMLElement | null
    isLastRow: boolean
    isSingleRow: boolean
  }>()

  // Keep emits, they will be forwarded from the child
  const emit = defineEmits(['deleted', 'delete-start', 'delete-end', 'sidebarClick'])

  // Add a ref to access the child component's exposed methods
  const deleteActionRef = ref<InstanceType<typeof DeleteDocumentAction> | null>(null)

  // --- Keep Scroll logic ---
  const scrollElement = ref<HTMLElement | Window>(window)
  const showBoom = ref(false)

  const updateScroll = () => {
    let scrollLeft = 0,
      scrollWidth = 0,
      clientWidth = 0
    const el = scrollElement.value
    if (el instanceof HTMLElement) {
      scrollLeft = el.scrollLeft
      scrollWidth = el.scrollWidth
      clientWidth = el.clientWidth
    } else {
      scrollLeft = window.scrollX
      scrollWidth = document.documentElement.scrollWidth
      clientWidth = window.innerWidth
    }
    const actionStart = scrollWidth - 30
    showBoom.value = scrollLeft + clientWidth < actionStart
  }

  const setupScrollListener = () => {
    const el = scrollElement.value
    const handler = updateScroll
    ;(el instanceof HTMLElement ? el : window).addEventListener('scroll', handler, {
      passive: true,
    })
    updateScroll()
  }

  const removeScrollListener = () => {
    const el = scrollElement.value
    ;(el instanceof HTMLElement ? el : window).removeEventListener('scroll', updateScroll)
  }
  // --- End Scroll logic ---

  // Method to trigger the delete dialog in the child component
  const triggerDelete = () => {
    deleteActionRef.value?.openDeleteDialog()
  }

  // REMOVE: openDeleteDialog, closeDialog, confirmDelete, handleKeyDown

  // --- Lifecycle Hooks for Scroll ---
  onMounted(() => {
    // REMOVE: document.addEventListener('keydown', handleKeyDown);
    if (props.targetRef) scrollElement.value = props.targetRef
    setupScrollListener()
    showBoom.value = false // Initialize showBoom state
  })

  onBeforeUnmount(() => {
    // REMOVE: document.removeEventListener('keydown', handleKeyDown);
  })

  onUnmounted(() => {
    removeScrollListener()
  })
  // --- End Lifecycle Hooks ---
</script>

<template>
  <div
    v-if="showBoom"
    class="bg-white pb-0 fixed right-0 transform -translate-y-1/2 bg-transparent boom-sidebar z-10"
  ></div>
  <div class="h-1">
    <TableCell
      v-if="showBoom"
      class="excel-delete-cell excel-cell border-t fixed right-0 select-none z-20"
      :class="{
        'border-b-[1px]': isSingleRow || isLastRow,
      }"
    >
      <Button
        variant="ghost"
        size="sm"
        class="excel-delete-button -pb-0 -mb-[10px]"
        @click="triggerDelete"
        :disabled="false"
      >
        <TrashIcon class="h-4 w-4" />
      </Button>
    </TableCell>
  </div>
  <DeleteDocumentAction
    ref="deleteActionRef"
    :collection-name="props.collectionName"
    :document-id="props.documentId"
    :row-number="props.rowNumber"
    @deleted="emit('deleted')"
    @delete-start="(id) => emit('delete-start', id)"
    @delete-end="emit('delete-end')"
  />
</template>

<style scoped>
  /* Keep styles for the button and sidebar */
  .excel-delete-button {
    position: relative; /* required for z-index to apply */
    color: #d32f2f;
  }
  .excel-delete-button:hover {
    color: #b71c1c;
    background-color: #ffebee;
  }
  .boom-sidebar {
    width: 57px;
    box-shadow: 0 0 1px rgba(0, 0, 0, 0.5);
    height: 100%;
  }

  /* REMOVE .custom-delete-dialog style */
</style>
