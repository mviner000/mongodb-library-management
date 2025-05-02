<script setup lang="ts">
  import { computed, defineProps, defineEmits, PropType } from 'vue'

  // UI Component Imports (same as parent)
  import { ReloadIcon, Cross2Icon } from '@radix-icons/vue'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { TableCell, TableRow } from '@/components/ui/table'
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { ScrollArea } from '@/components/ui/scroll-area'

  // Define Props
  const props = defineProps({
    tableHeaders: {
      type: Array as PropType<string[]>,
      required: true,
    },
    newDocument: {
      type: Object as PropType<Record<string, any>>,
      required: true,
    },
    isSaving: {
      type: Boolean,
      required: true,
    },
    errorColumn: {
      type: String as PropType<string | null>,
      default: null,
    },
    addingRowError: {
      type: Boolean,
      default: false,
    },
    highlightedColumn: {
      type: String as PropType<string | null>,
      default: null,
    },
    totalDocuments: {
      type: Number,
      required: true,
    },
    // Pass store helpers or necessary parts of schema/options
    getSchemaInfo: {
      type: Function as PropType<(field: string) => any>,
      required: true,
    },
    isReferenceField: {
      type: Function as PropType<(field: string) => boolean>,
      required: true,
    },
    getReferencedCollection: {
      type: Function as PropType<(field: string) => string | null>,
      required: true,
    },
    referenceOptions: {
      type: Object as PropType<Record<string, { id: string; label: string }[]>>,
      required: true,
    },
    loadingReferences: {
      type: Object as PropType<Record<string, boolean>>,
      required: true,
    },
    searchQuery: {
      // If search is needed within the add row itself
      type: Object as PropType<Record<string, string>>,
      required: true,
    },
  })

  // Define Emits
  const emit = defineEmits(['update:newDocument', 'save', 'cancel'])

  // Local computed property to proxy v-model for newDocument
  const localNewDocument = computed({
    get: () => props.newDocument,
    set: (value) => {
      emit('update:newDocument', value)
    },
  })

  // Local computed property for filtered options (copied from parent)
  const filteredOptions = (field: string) => {
    const refCollection = props.getReferencedCollection(field) // Use prop function
    if (!refCollection) return []
    const options = props.referenceOptions[refCollection] || [] // Use prop data
    const query = (props.searchQuery[field] || '').toLowerCase() // Use prop data
    return options.filter((opt) => opt.label.toLowerCase().includes(query))
  }

  // --- Event Handlers ---
  const handleSave = () => {
    emit('save')
  }

  const handleCancel = () => {
    emit('cancel')
  }
</script>

<template>
  <TableRow
    class="excel-new-row"
    :class="{ 'excel-new-row-error': addingRowError }"
  >
    <TableCell
      class="excel-column-checkbox-selector"
      :style="{
        width: '40px',
        minWidth: '40px',
        maxWidth: '40px',
      }"
    >
      <input
        type="checkbox"
        disabled
        class="excel-checkbox"
      />
    </TableCell>
    <TableCell class="excel-row-number"> {{ totalDocuments + 1 }} </TableCell>
    <TableCell
      v-for="header in tableHeaders"
      :key="`new-${header}`"
      class="excel-cell"
      :class="{
        'error-column-cell': header === errorColumn,
        'highlighted-column': highlightedColumn === header,
      }"
    >
      <span
        v-if="['_id', 'created_at', 'updated_at'].includes(header)"
        class="excel-timestamp excel-auto-id"
      >
        {{ header === '_id' ? '(auto)' : '(auto-generated)' }}
      </span>

      <div
        v-else-if="getSchemaInfo(header).bsonType === 'bool'"
        class="flex items-center justify-center"
      >
        <input
          type="checkbox"
          v-model="localNewDocument[header]"
          class="excel-checkbox"
        />
      </div>
      <div
        v-else-if="isReferenceField(header)"
        class="h-8"
      >
        <Select
          v-model="localNewDocument[header]"
          class="excel-select"
        >
          <SelectTrigger class="h-8">
            <SelectValue :placeholder="`Select`" />
          </SelectTrigger>
          <SelectContent>
            <div
              v-if="loadingReferences[getReferencedCollection(header)!]"
              class="p-2"
            >
              <ReloadIcon class="h-4 w-4 animate-spin mx-auto" />
            </div>
            <ScrollArea
              v-else
              class="h-48"
            >
              <SelectItem
                v-for="option in filteredOptions(header)"
                :key="option.id"
                :value="option.id"
              >
                {{ option.label }}
              </SelectItem>
              <div
                v-if="
                  !filteredOptions(header).length &&
                  !loadingReferences[getReferencedCollection(header)!]
                "
                class="text-sm text-gray-500 px-2 py-1"
              >
                No options found
              </div>
            </ScrollArea>
          </SelectContent>
        </Select>
      </div>
      <Input
        v-else
        v-model="localNewDocument[header]"
        :type="getSchemaInfo(header).bsonType === 'date' ? 'datetime-local' : 'text'"
        class="excel-input"
        :class="{ 'ring-2 ring-red-500 border-red-500': header === errorColumn }"
      />
    </TableCell>
    <TableCell class="excel-cell excel-actions-cell text-center">
      <Button
        variant="ghost"
        @click="handleSave"
        size="sm"
        class="px-1 h-full text-green-600 hover:text-green-800 disabled:text-gray-400"
        :disabled="isSaving"
        title="Save"
      >
        <ReloadIcon
          v-if="isSaving"
          class="h-4 w-4 animate-spin"
        />
        <span
          v-else
          class="text-lg"
          >💾</span
        >
      </Button>
      <Button
        variant="ghost"
        @click="handleCancel"
        size="sm"
        class="px-1 h-full text-red-500 hover:text-red-700 disabled:text-gray-400"
        :disabled="isSaving"
        title="Cancel"
      >
        <Cross2Icon class="h-4 w-4" />
      </Button>
    </TableCell>
  </TableRow>
</template>

<style scoped>
  /* Add the same scoped styles relevant to the new row from MongoDBDataTable.vue if needed */
  /* e.g., .excel-new-row, .excel-new-row-error, input styles */

  .excel-new-row {
    background-color: #e8f5e9; /* [cite: 387] */
  }
  .excel-new-row-error {
    background-color: #fee2e2 !important; /* [cite: 430] */
  }
  .excel-new-row .excel-cell {
    height: 40px; /* [cite: 388] */
    overflow: visible; /* [cite: 388] */ /* Allow select dropdown */
    vertical-align: middle; /* [cite: 389] */
  }
  .excel-new-row .excel-input {
    overflow: hidden; /* [cite: 390] */
    height: 100%; /* [cite: 391] */ /* Ensure input fills cell */
    background-color: white; /* [cite: 391] */
    border: 1px solid #d0d0d0; /* [cite: 391] */ /* Add border for clarity */
  }

  .excel-new-row .excel-select > button {
    /* Target trigger */
    height: 100%; /* [cite: 392] */
    background-color: white; /* [cite: 392] */
    border: 1px solid #d0d0d0; /* [cite: 392] */
  }

  .error-column-cell {
    /* Applied to TD in add row */
    background-color: #fef2f2 !important; /* [cite: 427] */
  }
  .excel-new-row .error-column-cell .excel-input {
    /* Target input in error cell */
    border: 1px solid #ef4444 !important; /* [cite: 429] */
    outline: 1px solid #ef4444 !important; /* [cite: 429] */
  }
  .excel-auto-id {
    color: #888888; /* [cite: 384] */
    font-style: italic; /* [cite: 384] */
    padding: 0 8px; /* [cite: 385] */
    font-size: 12px; /* [cite: 385] */
    display: block; /* [cite: 386] */
    line-height: normal; /* [cite: 386] */
  }
  .excel-timestamp {
    color: #666666; /* [cite: 380] */
    font-style: italic; /* [cite: 380] */
    font-size: 12px; /* [cite: 381] */
    display: block; /* [cite: 382] */
    line-height: normal; /* [cite: 383] */
  }

  /* Make sure checkbox style is included */
  .excel-checkbox {
    height: 16px; /* [cite: 375] */
    width: 16px; /* [cite: 375] */
    cursor: pointer; /* [cite: 376] */
    accent-color: #217346; /* [cite: 376] */
    vertical-align: middle; /* [cite: 377] */
  }

  /* Basic styles for select */
  .excel-select {
    font-size: 14px; /* [cite: 407] */
    width: 100%; /* [cite: 407] */
  }
  .excel-select > button {
    /* Trigger */
    height: 100%; /* [cite: 408] */
    border-radius: 0; /* [cite: 408] */
    border: none; /* [cite: 408] */
    padding: 6px 8px; /* [cite: 408] */
    box-sizing: border-box; /* [cite: 409] */
    justify-content: space-between; /* [cite: 409] */
    background-color: transparent; /* [cite: 410] */
  }
  .excel-new-row .excel-select > button {
    /* Add border for add row select */
    border: 1px solid #d0d0d0;
    background-color: white;
  }

  /* General input style */
  .excel-input {
    height: 100%; /* [cite: 365] */
    width: 100%; /* [cite: 365] */
    min-height: 32px; /* [cite: 366] */
    border-radius: 0; /* [cite: 367] */
    border: none; /* [cite: 367] */
    box-shadow: none; /* [cite: 368] */
    font-size: 14px; /* [cite: 368] */
    padding: 4px 6px; /* [cite: 369] */
    outline: none; /* [cite: 370] */
    box-sizing: border-box; /* [cite: 370] */
    background-color: white; /* [cite: 370] */
  }
  .excel-input:focus-visible {
    outline: none; /* [cite: 371] */
    box-shadow: none; /* [cite: 372] */
    border: none; /* [cite: 372] */
  }

  .excel-cell {
    border: 1px solid #d0d0d0; /* [cite: 340] */
    padding: 0; /* [cite: 340] */
    font-size: 14px; /* [cite: 341] */
    color: #212121; /* [cite: 341] */
    position: relative; /* [cite: 342] */
    height: 40px; /* [cite: 342] */
    box-sizing: border-box; /* [cite: 343] */
    vertical-align: middle; /* [cite: 343] */
    overflow: hidden; /* [cite: 344] */
    white-space: nowrap; /* [cite: 344] */
    text-overflow: ellipsis; /* [cite: 345] */
  }
  .excel-row-number {
    position: sticky; /* [cite: 310] */
    left: 40px; /* [cite: 310] */
    z-index: 2; /* [cite: 311] */
    background-color: #f3f3f3; /* [cite: 311] */
    border: 1px solid #d0d0d0; /* [cite: 312] */
    text-align: center; /* [cite: 313] */
    vertical-align: middle; /* [cite: 313] */
    box-sizing: border-box; /* [cite: 314] */
    width: 30px !important; /* [cite: 314] */
    min-width: 30px !important; /* [cite: 315] */
    max-width: 30px !important; /* [cite: 315] */
    font-size: 14px; /* [cite: 316] */
    color: #616161; /* [cite: 316] */
  }
  .excel-column-checkbox-selector {
    position: sticky; /* [cite: 291] */
    left: 0; /* [cite: 291] */
    z-index: 5; /* [cite: 292] */
    background-color: #f3f3f3; /* [cite: 292] */
    border: 1px solid #d0d0d0; /* [cite: 293] */
    text-align: center; /* [cite: 293] */
    vertical-align: middle; /* [cite: 294] */
    box-sizing: border-box; /* [cite: 294] */
  }
  .excel-actions-cell {
    border: 1px solid #d0d0d0; /* [cite: 331] */
    position: sticky; /* [cite: 331] */
    right: 0; /* [cite: 331] */
    z-index: 2; /* [cite: 332] */
    width: 60px !important; /* [cite: 332] */
    min-width: 60px !important; /* [cite: 333] */
    max-width: 60px !important; /* [cite: 333] */
    background-color: #ffffff; /* [cite: 334] */
    text-align: center; /* [cite: 334] */
    vertical-align: middle; /* [cite: 334] */
    box-sizing: border-box; /* [cite: 335] */
  }
  /* Highlighted column style */
  .highlighted-column {
    position: relative; /* [cite: 251] */
    outline: none; /* [cite: 251] */
    box-shadow: /* [cite: 251] */
      1px 0 0 0 #2196f3,
      -1px 0 0 0 #2196f3;
    z-index: 2; /* [cite: 252] */
    background-color: rgba(33, 150, 243, 0.05) !important; /* [cite: 252] */
  }
  .highlighted-column::after {
    content: ''; /* [cite: 253] */
    position: absolute; /* [cite: 253] */
    top: 0; /* [cite: 253] */
    bottom: 0; /* [cite: 253] */
    right: -2px; /* [cite: 253] */
    width: 2px; /* [cite: 254] */
    background-color: #2196f3; /* [cite: 254] */
    z-index: 3; /* [cite: 254] */
  }
</style>
