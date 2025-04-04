<!-- src/components/MongoDBDataTable.vue -->
<script setup lang="ts">
import { ref, computed, watch, onMounted, Ref, inject } from 'vue';
import { useRoute, useRouter } from 'vue-router';
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
import { ScrollArea } from '@/components/ui/scroll-area';
import { useToast } from '@/components/ui/toast/use-toast';
import { PlusCircledIcon } from '@radix-icons/vue';
import MongoDBTableNavbar from './MongoDBTableNavbar.vue';

const { toast } = useToast();
const route = useRoute();
const router = useRouter();

// Accept both direct prop and route param
const props = defineProps<{
  selectedCollection?: string;
  name?: string; // From route params
}>();

// Use route param or direct prop, with fallback to 'users'
const collectionName = ref(props.name || props.selectedCollection || 'users');
const documents = ref<any[]>([]);
const isLoading = ref(false);
const errorMessage = ref('');
const pageSize = ref(10);
const currentPage = ref(1);
const filterQuery = ref('{}');
const collectionSchema = ref<any>({});
const newDocument = ref<Record<string, any>>({});
const isAdding = ref(false);
const showSchemaAsRow = ref(true);

const API_BASE = 'http://localhost:3000';

// Reference handling
const referenceOptions = ref<Record<string, Array<{ id: string; label: string }>>>({});
const searchQuery = ref<Record<string, string>>({});
const loadingReferences = ref<Record<string, boolean>>({});

const isReferenceField = (field: string) => {
  return getSchemaInfo(field).description?.includes('REF:');
};

const getReferencedCollection = (field: string) => {
  const desc = getSchemaInfo(field).description || '';
  const match = desc.match(/REF:(\w+)/);
  return match ? match[1] : null;
};

async function fetchReferenceOptions(collectionName: string) {
  loadingReferences.value[collectionName] = true;
  try {
    const response = await fetch(`${API_BASE}/collections/${collectionName}/documents`);
    const { success, data, error } = await response.json();
    
    if (success) {
      const options = data.map((doc: any) => ({
        id: doc._id.$oid,
        label: doc.label || doc.name || doc._id.$oid,
      }));
      referenceOptions.value[collectionName] = options;
      
      if (options.length === 0) {
        toast({
          title: 'No options found',
          description: `No documents found in ${collectionName}. Please add some first.`,
          variant: 'destructive',
        });
      }
    } else {
      toast({
        title: 'Error fetching options',
        description: error || 'Failed to fetch reference data',
        variant: 'destructive',
      });
    }
  } catch (error) {
    toast({
      title: 'Error',
      description: `Failed to fetch ${collectionName}: ${error}`,
      variant: 'destructive',
    });
  } finally {
    loadingReferences.value[collectionName] = false;
  }
}

const filteredOptions = (field: string) => {
  const refCollection = getReferencedCollection(field);
  if (!refCollection) return [];
  const options = referenceOptions.value[refCollection] || [];
  const query = (searchQuery.value[field] || '').toLowerCase();
  return options.filter(opt => opt.label.toLowerCase().includes(query));
};

// Watch for collection name changes with immediate execution
watch(collectionName, async (newVal) => {
  try {
    // Update route if it doesn't match the current collection
    if (route.params.name !== newVal) {
      router.push(`/collection/${newVal}`);
    }
    
    const response = await fetch(`${API_BASE}/collections/${newVal}/schema`);
    if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
    const { success, data, error } = await response.json();
    
    if (success) {
      collectionSchema.value = data;
      newDocument.value = initializeNewDocument();
      await fetchDocuments();
      
      // Fetch reference data for this collection's schema
      if (data.properties) {
        Object.keys(data.properties).forEach((field) => {
          const refCollection = getReferencedCollection(field);
          if (refCollection && !referenceOptions.value[refCollection]) {
            fetchReferenceOptions(refCollection);
          }
        });
      }
    } else {
      errorMessage.value = error || 'Schema fetch failed';
    }
  } catch (error) {
    errorMessage.value = `Schema error: ${error}`;
  }
}, { immediate: true });

// Watch for route changes to update collection name
watch(() => route.params.name, (newVal) => {
  if (newVal && newVal !== collectionName.value) {
    collectionName.value = newVal as string;
  }
}, { immediate: true });

// Watch for prop changes from parent
watch(() => props.selectedCollection, (newVal) => {
  if (newVal && newVal !== collectionName.value) {
    collectionName.value = newVal;
  }
});

// Watch for adding state
watch(isAdding, async (newVal) => {
  if (newVal) {
    tableHeaders.value.forEach((field) => {
      const refCollection = getReferencedCollection(field);
      if (refCollection && !referenceOptions.value[refCollection]) {
        fetchReferenceOptions(refCollection);
      }
    });
  }
});

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

// function to resolve reference labels
const getReferenceLabel = (field: string, id: string) => {
  const refCollection = getReferencedCollection(field);
  if (!refCollection) return id;
  const options = referenceOptions.value[refCollection] || [];
  const option = options.find(opt => opt.id === id);
  return option ? option.label : id;
};

// Check if a field is required
const isFieldRequired = (field: string) => {
  return collectionSchema.value.required?.includes(field) || false;
};

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
    if (['created_at', 'updated_at'].includes(field)) return;
    const prop = properties[field];
    // Handle boolean initialization
    if (prop.bsonType === 'bool') {
      doc[field] = false;
    } else {
      doc[field] = getDefaultValue(prop.bsonType);
    }
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
    const response = await fetch(`${API_BASE}/collections/${collectionName.value}/documents`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newDocument.value)
    });

    const { success, error } = await response.json();
    
    if (success) {
      isAdding.value = false;
      await fetchDocuments();
    } else {
      errorMessage.value = error || 'Create failed';
    }
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

    // Build URL with URLSearchParams for the filter
    let url = `${API_BASE}/collections/${collectionName.value}/documents`;
    
    const params = new URLSearchParams();
    params.append('filter', JSON.stringify(filter));
    url += `?${params.toString()}`;
    
    const response = await fetch(url, {
      method: 'GET',
      headers: { 'Content-Type': 'application/json' },
    });

    if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
    const { success, data, error } = await response.json();

    if (success) {
      documents.value = data;
      currentPage.value = 1;
      console.log("Fetched documents:", data);
    } else {
      errorMessage.value = error || 'Failed to fetch documents';
      documents.value = [];
      console.log("Error from API:", error);
    }
  } catch (error) {
    errorMessage.value = `Error fetching documents: ${error}`;
    documents.value = [];
    console.error("Exception:", error);
  } finally {
    isLoading.value = false;
  }
};

const handleCellClick = (rowIndex: number, header: string, value: any) => {
  if (['_id', 'created_at', 'updated_at'].includes(header)) return;
  editingCell.value = { rowIndex, header };
  editValue.value = typeof value === 'object' ? JSON.stringify(value, null, 2) : String(value);
};

const saveEdit = async () => {
  if (!editingCell.value || isSaving.value) return;
  isSaving.value = true;

  try {
    const doc = paginatedDocuments.value[editingCell.value.rowIndex];
    
    // Handle reference fields differently
    let parsedValue: any;
    if (isReferenceField(editingCell.value.header)) {
      parsedValue = editValue.value;
    } else {
      const fieldInfo = getSchemaInfo(editingCell.value.header);
      const bsonType = Array.isArray(fieldInfo.bsonType) 
        ? fieldInfo.bsonType[0] 
        : fieldInfo.bsonType;
      
      // Handle string fields without JSON parsing
      if (bsonType === 'string') {
        parsedValue = editValue.value;
      } else {
        parsedValue = JSON.parse(editValue.value);
      }
    }

    const update = { [editingCell.value.header]: parsedValue };

    const response = await fetch(
      `${API_BASE}/collections/${collectionName.value}/documents/${doc._id.$oid}`,
      {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(update)
      }
    );

    const { success, data, error } = await response.json();
    
    if (success && data.modified_count > 0) {
      // Refresh the updated document
      const filter = JSON.stringify({ _id: { $oid: doc._id.$oid } });
      const response = await fetch(
        `${API_BASE}/collections/${collectionName.value}/documents?filter=${encodeURIComponent(filter)}`
      );
      const { success: fetchSuccess, data: fetchData } = await response.json();

      if (fetchSuccess && fetchData.length > 0) {
        const index = documents.value.findIndex(d => d._id.$oid === doc._id.$oid);
        if (index !== -1) {
          documents.value.splice(index, 1, fetchData[0]);
          documents.value = [...documents.value];
        }
      }
      editingCell.value = null;
    } else {
      errorMessage.value = error || 'Update failed';
    }
  } catch (error) {
    errorMessage.value = `Error updating field: ${error}`;
  } finally {
    isSaving.value = false;
  }
};

const deleteDocument = async (id: string) => {
  if (!confirm('Are you sure you want to delete this document?')) return;
  try {
    const response = await fetch(
      `${API_BASE}/collections/${collectionName.value}/documents/${id}`,
      { method: 'DELETE' }
    );

    const { success, data, error } = await response.json();
    
    if (success && data.deleted_count > 0) {
      await fetchDocuments();
    } else {
      errorMessage.value = error || 'Delete failed';
    }
  } catch (error) {
    errorMessage.value = `Error deleting document: ${error}`;
  }
};

const collectionsList = ref<string[]>([]);
const fetchCollections = async () => {
  try {
    const response = await fetch(`${API_BASE}/collections`);
    if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
    const { success, data, error } = await response.json();
    
    if (success) {
      collectionsList.value = data;
      if (data.length > 0 && !data.includes(collectionName.value)) {
        // If current collection not in list, select first
        collectionName.value = data[0];
      }
    } else {
      errorMessage.value = error || 'Failed to fetch collections';
    }
  } catch (error) {
    errorMessage.value = `Error fetching collections: ${error}`;
  }
};

onMounted(() => {
  fetchCollections();
});

defineExpose({ 
  fetchDocuments, 
  fetchCollections,
  setCollection: (name: string) => {
    collectionName.value = name;
  }
});

const onPageChange = (page: number) => {
  currentPage.value = page;
};

const isSplit = inject<Ref<boolean>>('isSplit')!; // Inject isSplit from App.vue

// We already have a watch on collectionName that calls fetchDocuments
</script>
<template>
  <MongoDBTableNavbar 
    :isSplitActive="isSplit" 
    class="sticky top-0 z-50" 
  />
  <div class="border rounded-md p-4 w-full">
    
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
                        
                        <div v-if="isReferenceField(field)">References:</div>
                        <div v-if="isReferenceField(field)" class="font-medium">{{ getReferencedCollection(field) }}</div>
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
  
      <!-- Add document form (updated) -->
      <div v-if="isAdding" class="mt-6 border rounded-md p-4 bg-gray-50">
        <h3 class="text-lg font-semibold mb-4">Add New Document</h3>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div v-for="field in tableHeaders" :key="`new-${field}`" class="mb-4">
            <Label :for="`field-${field}`" class="block mb-1">
              {{ field }}
              <span v-if="isFieldRequired(field)" class="text-red-500">*</span>
            </Label>
            
            <!-- Boolean field handling -->
            <template v-if="getSchemaInfo(field).bsonType === 'bool'">
              <div class="flex items-center gap-2">
                <input 
                  type="checkbox" 
                  :id="`field-${field}`"
                  v-model="newDocument[field]"
                  class="h-4 w-4"
                />
                <label :for="`field-${field}`" class="text-sm">
                  {{ newDocument[field] ? 'True' : 'False' }}
                </label>
              </div>
            </template>
            
            <!-- Reference field with dropdown -->
            <template v-else-if="isReferenceField(field)">
              <div v-if="loadingReferences[getReferencedCollection(field)]" class="flex items-center gap-2">
                <ReloadIcon class="h-4 w-4 animate-spin" />
                <span>Loading options...</span>
              </div>
              <Select v-else v-model="newDocument[field]">
                <SelectTrigger>
                  <SelectValue :placeholder="`Select ${getReferencedCollection(field)}`" />
                </SelectTrigger>
                <SelectContent>
                  <ScrollArea class="h-48">
                    <div class="p-1">
                      <Input 
                        v-model="searchQuery[field]"
                        placeholder="Search..."
                        class="mb-2"
                      />
                      <div v-if="filteredOptions(field).length">
                        <SelectItem 
                          v-for="option in filteredOptions(field)"
                          :key="option.id"
                          :value="option.id"
                        >
                          {{ option.label }}
                        </SelectItem>
                      </div>
                      <div v-else class="text-sm text-gray-500 px-2 py-1">
                        No options found
                      </div>
                    </div>
                  </ScrollArea>
                </SelectContent>
              </Select>
            </template>

            <!-- Regular field -->
            <template v-else>
              <Input 
                v-if="field !== '_id'"
                :id="`field-${field}`"
                v-model="newDocument[field]"
                :type="collectionSchema.properties[field]?.bsonType === 'date' ? 'datetime-local' : 'text'"
                :class="{ 'border-red-300': isFieldRequired(field) && !newDocument[field] }"
              />
              <span v-else class="text-gray-400">(auto-generated)</span>
            </template>
            
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
          <TableRow v-if="showSchemaAsRow" class="bg-blue-50 text-xs hidden">
            <TableCell v-for="header in tableHeaders" :key="`schema-${header}`" 
              class="border-r border-b border-gray-200 p-2">
              <div class="flex flex-col">
                <span class="font-semibold">{{ header }}</span>
                <span class="text-gray-600">{{ getFieldTypeName(header) }}</span>
                <span class="text-gray-600">{{ getSchemaInfo(header).description }}</span>
                <span v-if="isReferenceField(header)" class="text-blue-600">
                  References: {{ getReferencedCollection(header) }}
                </span>
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
                  <!-- Boolean field editing -->
                  <div v-if="collectionSchema.properties[header]?.bsonType === 'bool'" 
                    class="flex items-center justify-center h-full p-2">
                    <input 
                      type="checkbox" 
                      v-model="editValue" 
                      @change="saveEdit"
                      class="h-4 w-4"
                    />
                  </div>
                  <!-- Reference field editing -->
                  <div v-else-if="isReferenceField(header)" class="p-1">
                    <Select v-model="editValue" @update:modelValue="saveEdit">
                      <SelectTrigger>
                        <SelectValue :placeholder="`Select ${getReferencedCollection(header)}`" :model-value="editValue" />
                      </SelectTrigger>
                      <SelectContent>
                        <ScrollArea class="h-48">
                          <div class="p-1">
                            <Input 
                              v-model="searchQuery[header]"
                              placeholder="Search..."
                              class="mb-2"
                            />
                  <div v-if="filteredOptions(header).length">
                    <SelectItem 
                      v-for="option in filteredOptions(header)"
                      :key="option.id"
                      :value="option.id"
                    >
                      {{ option.label }}
                    </SelectItem>
                  </div>
                  <div v-else class="text-sm text-gray-500 px-2 py-1">
                    No options found
                  </div>
                </div>
              </ScrollArea>
            </SelectContent>
          </Select>
        </div>
        <!-- Date input for date fields -->
        <Input v-else-if="collectionSchema.properties[header]?.bsonType === 'date'"
          type="datetime-local"
          v-model="editValue"
          @blur="saveEdit"
          class="h-full rounded-none border-none focus-visible:ring-2"
        />
        <!-- Default textarea for other fields -->
        <textarea v-else
          v-model="editValue"
          @blur="saveEdit"
          @keyup.ctrl.enter="saveEdit"
          class="w-full h-full p-2 font-mono text-sm border-none focus:ring-2 focus:ring-blue-500"
          rows="3"
        />
      </div>
      <div v-else
        class="p-2 min-h-[40px]"
        :class="{
          'cursor-pointer hover:bg-blue-50': !['created_at', 'updated_at'].includes(header),
          'cursor-not-allowed': ['created_at', 'updated_at'].includes(header)
        }"
        @click="!['created_at', 'updated_at', '_id'].includes(header) && handleCellClick(rowIndex, header, doc[header])"
      >
        <!-- Show boolean values with checkboxes in read-only mode -->
        <div v-if="collectionSchema.properties[header]?.bsonType === 'bool'" class="flex justify-center">
          <input type="checkbox" :checked="doc[header]" disabled class="h-4 w-4" />
        </div>
        <!-- Show reference field labels in read-only mode -->
        <div v-else-if="isReferenceField(header)" class="text-blue-600">
          {{ getReferenceLabel(header, doc[header]) || doc[header] }}
        </div>
        <!-- Add disabled display for timestamp fields -->
        <template v-else-if="['created_at', 'updated_at'].includes(header)">
          <span class="text-gray-500">
            {{ formatSchemaValue(doc[header], collectionSchema.properties[header]?.bsonType) }}
          </span>
        </template>
        <template v-else>
          {{ formatSchemaValue(doc[header], collectionSchema.properties[header]?.bsonType) }}
        </template>
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
              <!-- Boolean field in inline add mode -->
              <div v-if="header !== '_id' && collectionSchema.properties[header]?.bsonType === 'bool'" 
                class="flex items-center justify-center">
                <input 
                  type="checkbox" 
                  v-model="newDocument[header]"
                  class="h-4 w-4"
                />
              </div>
              <!-- Reference field in inline add mode -->
              <div v-else-if="header !== '_id' && isReferenceField(header)" class="h-8">
                <Select v-model="newDocument[header]" class="h-8">
                  <SelectTrigger class="h-8">
                    <SelectValue :placeholder="`Select`" />
                  </SelectTrigger>
                  <SelectContent>
                    <div v-if="loadingReferences[getReferencedCollection(header)]" class="p-2">
                      <ReloadIcon class="h-4 w-4 animate-spin mx-auto" />
                    </div>
                    <ScrollArea v-else class="h-48">
                      <Input 
                        v-model="searchQuery[header]"
                        placeholder="Search..."
                        class="mb-1 mx-1"
                      />
                      <SelectItem 
                        v-for="option in filteredOptions(header)"
                        :key="option.id"
                        :value="option.id"
                      >
                        {{ option.label }}
                      </SelectItem>
                    </ScrollArea>
                  </SelectContent>
                </Select>
              </div>
              <!-- Regular field in inline add mode -->
              <Input v-else-if="header !== '_id'"
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
.checkbox-cell {
  vertical-align: middle;
  text-align: center;
}

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

.cursor-not-allowed {
  cursor: not-allowed;
  opacity: 0.8;
}
</style>