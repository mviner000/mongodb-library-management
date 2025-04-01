<!-- src/components/MongoDBDataTable.vue -->
<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ReloadIcon, TrashIcon } from '@radix-icons/vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow
} from '@/components/ui/table';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Pagination,
  PaginationList,
  PaginationListItem,
  PaginationFirst,
  PaginationLast,
  PaginationNext,
  PaginationPrev,
} from '@/components/ui/pagination';

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger
} from '@/components/ui/tooltip';

import { PlusCircledIcon } from '@radix-icons/vue';
import { useClipboard } from '@vueuse/core';

const collectionName = ref('users');
const documents = ref<any[]>([]);
const isLoading = ref(false);
const errorMessage = ref('');
const pageSize = ref(10);
const currentPage = ref(1);
const filterQuery = ref('{}');
const collectionSchema = ref<any>({});
const newDocument = ref<Record<string, any>>({});
const isAdding = ref(false);
const { copy } = useClipboard();
const showSchemaAsRow = ref(true);

// Inline editing states
const editingCell = ref<{rowIndex: number; header: string} | null>(null);
const editValue = ref('');
const isSaving = ref(false);

// For pagination
const totalPages = computed(() => Math.ceil(documents.value.length / pageSize.value));
const paginatedDocuments = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return documents.value.slice(start, end);
});

// Get all unique keys from all documents for table headers
const tableHeaders = computed(() => {
  if (!collectionSchema.value.properties) return [];
  const props = collectionSchema.value.properties;
  return Object.keys(props).sort((a, b) => {
    // Sort required fields first
    const required = collectionSchema.value.required || [];
    if (required.includes(a) && !required.includes(b)) return -1;
    if (!required.includes(a) && required.includes(b)) return 1;
    return a.localeCompare(b);
  });
});

// Check if schema is available
const hasSchema = computed(() => {
  return collectionSchema.value && collectionSchema.value.properties && Object.keys(collectionSchema.value.properties).length > 0;
});

// Get schema information for a specific field
const getSchemaInfo = (field: string) => {
  if (!collectionSchema.value.properties || !collectionSchema.value.properties[field]) return {};
  return collectionSchema.value.properties[field];
};

// Check if a field is required
const isFieldRequired = (field: string) => {
  return collectionSchema.value.required?.includes(field) || false;
};

function formatValue(value: any): string {
  if (value === undefined || value === null) return '';
  return typeof value === 'object' ? JSON.stringify(value) : String(value);
}

// Move formatSchemaValue into the setup scope
const formatSchemaValue = (value: any, bsonType: string | string[]) => {
  if (!value) return '';
  const type = Array.isArray(bsonType) ? bsonType[0] : bsonType;
  
  if (type === 'date' && value instanceof Date) {
    return value.toLocaleString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value, null, 2);
  }
  return String(value);
};

// Get friendly field type name
const getFieldTypeName = (field: string) => {
  const fieldInfo = getSchemaInfo(field);
  if (!fieldInfo || !fieldInfo.bsonType) return 'Unknown';
  
  const bsonType = Array.isArray(fieldInfo.bsonType) ? fieldInfo.bsonType[0] : fieldInfo.bsonType;
  
  switch(bsonType) {
    case 'string': return 'Text';
    case 'int': return 'Integer';
    case 'double': return 'Number';
    case 'bool': return 'Boolean';
    case 'date': return 'Date';
    case 'array': return 'Array';
    case 'object': return 'Object';
    default: return bsonType;
  }
};

watch(collectionName, async (newVal) => {
  try {
    console.log(`Fetching schema for collection: ${newVal}`);
    collectionSchema.value = await invoke('get_collection_schema', { 
      collectionName: newVal 
    });
    console.log('Schema fetch successful:', collectionSchema.value);
    newDocument.value = initializeNewDocument();
  } catch (error) {
    console.error('Schema fetch error:', error);
    errorMessage.value = `Schema error: ${error}`;
  }
});

const initializeNewDocument = () => {
  const doc: Record<string, any> = {};
  const required = collectionSchema.value.required || [];
  const properties = collectionSchema.value.properties || {};
  
  required.forEach((field: string) => {
    const prop = properties[field];
    doc[field] = getDefaultValue(prop.bsonType);
  });
  
  // Add required timestamps
  if (properties.created_at) doc.created_at = new Date();
  if (properties.updated_at) doc.updated_at = new Date();
  
  return doc;
};

const getDefaultValue = (bsonType: string | string[]) => {
  const type = Array.isArray(bsonType) ? bsonType[0] : bsonType;
  switch (type) {
    case 'string': return '';
    case 'int': case 'double': return 0;
    case 'bool': return false;
    case 'date': return new Date();
    case 'object': return {};
    case 'array': return [];
    default: return null;
  }
};

const startAdding = () => {
  isAdding.value = true;
  newDocument.value = initializeNewDocument();
};

const cancelAdding = () => {
  isAdding.value = false;
  newDocument.value = {};
};

const saveNewDocument = async () => {
  try {
    // Convert dates to timestamps
    Object.entries(collectionSchema.value.properties || {}).forEach(([key, prop]: [string, any]) => {
      if (prop.bsonType === 'date' && newDocument.value[key] instanceof Date) {
        newDocument.value[key] = newDocument.value[key].getTime();
      }
    });

    await invoke('insert_document', {
      collectionName: collectionName.value,
      document: newDocument.value
    });
    
    isAdding.value = false;
    await fetchDocuments();
  } catch (error) {
    errorMessage.value = `Create failed: ${error}`;
  }
};


const fetchDocuments = async () => {
  isLoading.value = true;
  errorMessage.value = '';
  try {
    let filter = {};
    try {
      filter = JSON.parse(filterQuery.value);
    } catch (error) {
      errorMessage.value = `Invalid filter JSON: ${error}`;
      return;
    }
    documents.value = await invoke<any[]>('find_documents', {
      collectionName: collectionName.value,
      filter: filter
    });
    currentPage.value = 1;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (message.includes('Database connection not initialized')) {
      setTimeout(fetchDocuments, 1000);
    } else {
      errorMessage.value = `Error fetching documents: ${message}`;
      documents.value = [];
    }
  } finally {
    isLoading.value = false;
  }
};

const handleCellClick = (rowIndex: number, header: string, value: any) => {
  if (header === '_id') return;
  editingCell.value = { rowIndex, header };
  editValue.value = typeof value === 'object' ? JSON.stringify(value, null, 2) : String(value);
};

const saveEdit = async () => {
  if (!editingCell.value || isSaving.value) return;
  isSaving.value = true;
  
  const { rowIndex, header } = editingCell.value;
  const doc = paginatedDocuments.value[rowIndex];
  const originalDoc = documents.value.find(d => d._id.$oid === doc._id.$oid);
  if (!originalDoc) return;

  try {
    // Parse value based on original type
    let newValue: any = editValue.value;
    if (typeof originalDoc[header] === 'number') {
      newValue = parseFloat(editValue.value);
    } else if (typeof originalDoc[header] === 'object') {
      newValue = JSON.parse(editValue.value);
    }

    const update = { [header]: newValue };
    const success = await invoke<boolean>('update_document', {
      collectionName: collectionName.value,
      id: doc._id.$oid,
      update: update
    });

    if (success) {
      originalDoc[header] = newValue;
      documents.value = [...documents.value];
    }
    editingCell.value = null;
  } catch (error) {
    errorMessage.value = `Error updating field: ${error}`;
  } finally {
    isSaving.value = false;
  }
};

const deleteDocument = async (id: string) => {
  if (!confirm('Are you sure you want to delete this document?')) return;
  try {
    const success = await invoke<boolean>('delete_document', {
      collectionName: collectionName.value,
      id: id
    });
    if (success) fetchDocuments();
  } catch (error) {
    errorMessage.value = `Error deleting document: ${error}`;
  }
};

const collectionsList = ref<string[]>([]);
const fetchCollections = async () => {
  try {
    collectionsList.value = await invoke<string[]>('list_collections');
    if (collectionsList.value.length > 0 && !collectionsList.value.includes(collectionName.value)) {
      collectionName.value = collectionsList.value[0];
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (message.includes('Database connection not initialized')) {
      setTimeout(fetchCollections, 1000);
    } else {
      errorMessage.value = `Error fetching collections: ${message}`;
    }
  }
};

onMounted(() => {
  fetchCollections();
});

defineExpose({ fetchDocuments, fetchCollections });

const onPageChange = (page: number) => {
  currentPage.value = page;
};

const toggleSchemaDisplay = () => {
  showSchemaAsRow.value = !showSchemaAsRow.value;
};

watch(collectionName, fetchDocuments);
</script>

<template>
  <div class="border rounded-md p-4 w-full">
    <h2 class="text-xl font-bold mb-4">MongoDB Data Table</h2>
    
    <div class="flex flex-col md:flex-row gap-4 mb-4">
      <div class="w-full md:w-1/4">
        <Label for="collection-select">Collection</Label>
        <Select v-model="collectionName">
          <SelectTrigger class="mt-1">
            <SelectValue placeholder="Select a collection" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="collection in collectionsList" :key="collection" :value="collection">
              {{ collection }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      
      <div class="w-full md:w-3/4">
        <Label for="filter-input">Filter Query (JSON)</Label>
        <div class="flex gap-2 mt-1">
          <Input id="filter-input" v-model="filterQuery" placeholder="{}" />
          <Button @click="fetchDocuments" :disabled="isLoading">
            <ReloadIcon v-if="isLoading" class="mr-2 h-4 w-4 animate-spin" />
            <span v-else>Filter</span>
          </Button>
        </div>
      </div>
    </div>
    
    <div class="flex justify-between items-center mb-2">
      <div class="flex items-center gap-2">
        <Button 
          v-if="documents.length > 0"
          variant="outline" 
          size="sm" 
          @click="toggleSchemaDisplay"
          class="text-xs"
        >
          {{ showSchemaAsRow ? 'Hide Schema Row' : 'Show Schema Row' }}
        </Button>
      </div>
    </div>
    
    <div v-if="errorMessage" class="my-2 p-2 bg-red-100 text-red-700 rounded">
      {{ errorMessage }}
    </div>
    
    <div v-if="isLoading" class="flex justify-center my-8">
      <ReloadIcon class="h-8 w-8 animate-spin text-gray-500" />
    </div>
    
    <!-- Schema info table when no documents found -->
    <div v-else-if="documents.length === 0 && hasSchema" class="w-full">
      <div class="mb-4">
    <h3 class="text-lg font-semibold mb-2">Collection Schema: {{ collectionName }}</h3>
    <p class="text-gray-600 mb-4">
      No documents found in this collection. Below is the schema definition.
    </p>
    
    <!-- Excel-style schema fields with tooltips -->
    <div class="overflow-x-auto pb-2 mb-4">
      <div class="flex excel-headers">
        <div v-for="field in tableHeaders" :key="field" class="relative">
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <div class="excel-header-cell" :class="isFieldRequired(field) ? 'excel-header-required' : ''">
                  {{ field }}
                  <span v-if="isFieldRequired(field)" class="text-red-500 ml-1">*</span>
                </div>
              </TooltipTrigger>
              <TooltipContent class="w-64 p-0">
                <div class="text-gray-600 bg-white rounded shadow border border-gray-200 p-3">
                  <div class="font-semibold border-b pb-1 mb-2 text-slate-600">{{ field }}</div>
                  <div class="grid grid-cols-2 gap-1 text-sm">
                    <div>Type:</div>
                    <div class="font-medium">{{ getFieldTypeName(field) }}</div>
                    
                    <div>Required:</div>
                    <div>
                      <span v-if="isFieldRequired(field)" class="text-green-600 font-medium">Yes</span>
                      <span v-else class="text-gray-500">No</span>
                    </div>
                    
                    <div>Description:</div>
                    <div>{{ getSchemaInfo(field).description || 'No description' }}</div>
                  </div>
                </div>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      </div>
    </div>
        
    <div class="mt-6">
      <Button @click="startAdding" class="gap-2">
        <PlusCircledIcon class="h-4 w-4" />
        <span>Add First Document</span>
      </Button>
    </div>
      </div>
  
  <!-- Add document form (unchanged) -->
  <div v-if="isAdding" class="mt-6 border rounded-md p-4 bg-gray-50">
    <h3 class="text-lg font-semibold mb-4">Add New Document</h3>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div v-for="field in tableHeaders" :key="`new-${field}`" class="mb-4">
        <Label :for="`field-${field}`" class="block mb-1">
          {{ field }}
          <span v-if="isFieldRequired(field)" class="text-red-500">*</span>
        </Label>
        <Input 
          v-if="field !== '_id'"
          :id="`field-${field}`"
          v-model="newDocument[field]"
          :type="collectionSchema.properties[field]?.bsonType === 'date' ? 'datetime-local' : 'text'"
          :class="{ 'border-red-300': isFieldRequired(field) && !newDocument[field] }"
        />
        <span v-else class="text-gray-400">(auto-generated)</span>
        <p class="text-xs text-gray-500 mt-1">
          {{ getSchemaInfo(field).description || '' }}
        </p>
      </div>
    </div>
    <div class="flex justify-end gap-2 mt-4">
      <Button @click="cancelAdding" variant="outline">Cancel</Button>
      <Button @click="saveNewDocument">Save Document</Button>
    </div>
  </div>
</div>
    
    <!-- No documents and no schema -->
    <div v-else-if="documents.length === 0 && !hasSchema" class="text-center my-8 text-gray-500">
      No documents found in collection "{{ collectionName }}" and no schema available.
      <div class="mt-4">
        <Button @click="startAdding" class="gap-2">
          <PlusCircledIcon class="h-4 w-4" />
          <span>Add First Document</span>
        </Button>
      </div>
    </div>
    
    <!-- Regular data table -->
    <div v-else class="w-full overflow-auto">
      <Table class="excel-style-table">
        <TableHeader>
          <TableRow>
            <TableHead v-for="header in tableHeaders" :key="header" 
              class="bg-gray-100 font-semibold border-r border-b border-gray-300">
              {{ header }}
              <span v-if="isFieldRequired(header)" class="text-red-500 ml-1">*</span>
            </TableHead>
            <TableHead class="bg-gray-100 font-semibold border-b border-gray-300 text-right">
              Actions
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <!-- Schema Info Row -->
          <TableRow v-if="showSchemaAsRow" class="bg-blue-50 text-xs">
            <TableCell v-for="header in tableHeaders" :key="`schema-${header}`" 
              class="border-r border-b border-gray-200 p-2">
              <div class="flex flex-col">
                <span class="font-semibold">{{ header }}</span>
                <span class="text-gray-600">{{ getFieldTypeName(header) }}</span>
                <span class="text-gray-600">{{ getSchemaInfo(header).description }}</span>
              </div>
            </TableCell>
            <TableCell class="border-b border-gray-200 p-2 text-right">
              <span class="text-xs text-gray-500">Schema Info</span>
            </TableCell>
          </TableRow>
          
          <!-- Regular Data Rows -->
          <TableRow v-for="(doc, rowIndex) in paginatedDocuments" :key="rowIndex">
            <TableCell v-for="header in tableHeaders" :key="`${rowIndex}-${header}`" 
            class="border-r border-b border-gray-200 p-0">
            <div class="h-full">
              <div v-if="editingCell?.rowIndex === rowIndex && editingCell?.header === header" 
                class="h-full">
                <!-- Add date input for date fields -->
                <Input v-if="collectionSchema.properties[header]?.bsonType === 'date'"
                  type="datetime-local"
                  v-model="editValue"
                  @blur="saveEdit"
                  class="h-full rounded-none border-none focus-visible:ring-2"
                />
                  <textarea v-else
                    v-model="editValue"
                    @blur="saveEdit"
                    @keyup.ctrl.enter="saveEdit"
                    class="w-full h-full p-2 font-mono text-sm border-none focus:ring-2 focus:ring-blue-500"
                    rows="3"
                  />
                </div>
                <div v-else
                    class="p-2 cursor-pointer hover:bg-blue-50 min-h-[40px]"
                    @click="handleCellClick(rowIndex, header, doc[header])"
                  >
                  {{ formatSchemaValue(doc[header], collectionSchema.properties[header]?.bsonType) }}
                </div>
              </div>
            </TableCell>
            <TableCell class="border-b border-gray-200 text-right p-1">
              <Button
                variant="ghost"
                size="sm"
                class="text-red-600 hover:text-red-800"
                @click="deleteDocument(doc._id.$oid)"
              >
                <TrashIcon class="h-4 w-4" />
              </Button>
            </TableCell>
          </TableRow>

          <TableRow v-if="isAdding" class="bg-blue-50">
            <TableCell v-for="header in tableHeaders" :key="header" class="p-1">
              <Input v-if="header !== '_id'"
                v-model="newDocument[header]"
                :type="collectionSchema.properties[header]?.bsonType === 'date' ? 'datetime-local' : 'text'"
                class="h-8"
              />
              <span v-else class="px-2 text-gray-400">(auto)</span>
            </TableCell>
            <TableCell class="space-x-1">
              <Button size="sm" @click="saveNewDocument">Save</Button>
              <Button size="sm" variant="ghost" @click="cancelAdding">Cancel</Button>
            </TableCell>
          </TableRow>

          <TableRow v-if="!isAdding" class="hover:bg-gray-50 cursor-pointer" 
            @click="startAdding">
            <TableCell :colspan="tableHeaders.length + 1" class="text-center py-2">
              <div class="inline-flex items-center gap-2 text-blue-600">
                <PlusCircledIcon class="h-4 w-4" />
                <span class="text-sm">Add new document</span>
              </div>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
      
      <div v-if="totalPages > 1" class="mt-4">
        <Pagination :page="currentPage" :itemsPerPage="pageSize" :total="documents.length"
          @update:page="onPageChange" :siblingCount="1">
          <PaginationList>
            <PaginationListItem :value="1">
              <PaginationFirst :disabled="currentPage === 1" @click="currentPage = 1" />
            </PaginationListItem>
            <PaginationListItem :value="Math.max(1, currentPage - 1)">
              <PaginationPrev :disabled="currentPage === 1" 
                @click="currentPage = Math.max(1, currentPage - 1)" />
            </PaginationListItem>
            <PaginationListItem :value="Math.min(totalPages, currentPage + 1)">
              <PaginationNext :disabled="currentPage === totalPages" 
                @click="currentPage = Math.min(totalPages, currentPage + 1)" />
            </PaginationListItem>
            <PaginationListItem :value="totalPages">
              <PaginationLast :disabled="currentPage === totalPages" 
                @click="currentPage = totalPages" />
            </PaginationListItem>
          </PaginationList>
        </Pagination>
      </div>
      
      <div class="flex items-center gap-2 mt-4">
        <span class="text-sm text-gray-500">Rows per page:</span>
        <Select v-model="pageSize">
          <SelectTrigger class="w-16">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="5">5</SelectItem>
            <SelectItem value="10">10</SelectItem>
            <SelectItem value="20">20</SelectItem>
            <SelectItem value="50">50</SelectItem>
            <SelectItem value="100">100</SelectItem>
          </SelectContent>
        </Select>
        
        <span class="ml-4 text-sm text-gray-500">
          Showing {{ (currentPage - 1) * pageSize + 1 }} to 
          {{ Math.min(currentPage * pageSize, documents.length) }} of 
          {{ documents.length }} entries
        </span>
      </div>
    </div>
  </div>
</template>

<style>
.excel-style-table {
  border-collapse: collapse;
  border-spacing: 0;
}

.excel-style-table th,
.excel-style-table td {
  @apply border-gray-300;
  border-style: solid;
  border-width: 0 1px 1px 0;
}

.excel-style-table th {
  @apply bg-gray-100;
}

.excel-style-table td {
  @apply bg-white;
}

.excel-style-table tr:hover td {
  @apply bg-gray-50;
}

.excel-style-table input {
  @apply h-full min-h-[40px] rounded-none border-none shadow-none focus-visible:ring-2;
}

.excel-style-table textarea {
  @apply min-h-[80px] resize-y;
}

.schema-table {
  @apply w-full border-collapse;
}

.schema-table th {
  @apply bg-gray-100 text-left p-2 border border-gray-300;
}

.schema-table td {
  @apply p-2 border border-gray-300;
}

.schema-table tr:hover td {
  @apply bg-gray-50;
}
</style>