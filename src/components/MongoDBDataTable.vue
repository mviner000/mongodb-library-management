<!-- src/components/MongoDBDataTable.vue -->
<script setup lang="ts">
import { ref, computed, watch, onMounted, Ref, inject } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { ReloadIcon, TrashIcon } from '@radix-icons/vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
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

import { getApiBaseUrl } from '@/utils/api';
import { useDebounceFn } from '@vueuse/core';
const API_BASE = getApiBaseUrl();

// Reference handling
const referenceOptions = ref<Record<string, Array<{ id: string; label: string }>>>({});
const searchQuery = ref<Record<string, string>>({});
const loadingReferences = ref<Record<string, boolean>>({});

const resizingState = ref({
  isResizing: false,
  header: '',
  startX: 0,
  startWidth: 0,
  currentWidth: 0 // Added for visual feedback during resize
});

// Add resize handlers
const startResize = (header: string, event: MouseEvent) => {
  console.log(`Starting resize for column: ${header}`);
  
  // Get current width from columnWidths or default
  const currentWidth = columnWidths.value[header] || 200;
  
  resizingState.value = {
    isResizing: true,
    header,
    startX: event.clientX,
    startWidth: currentWidth,
    currentWidth: currentWidth
  };
  
  // Add global event listeners
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopResize);
  
  // Prevent text selection during resize
  event.preventDefault();
};

const handleMouseMove = (event: MouseEvent) => {
  if (!resizingState.value.isResizing) return;
  
  const delta = event.clientX - resizingState.value.startX;
  const newWidth = Math.max(50, resizingState.value.startWidth + delta);
  
  // Update the current width for visual feedback
  resizingState.value.currentWidth = newWidth;
  
  // Update local schema
  const updatedWidths = {
    ...columnWidths.value,
    [resizingState.value.header]: newWidth
  };
  
  // Update the UI schema with new widths
  collectionSchema.value = {
    ...collectionSchema.value,
    ui: {
      ...collectionSchema.value.ui || {},
      columnWidths: updatedWidths
    }
  };
  
  console.log(`Resizing column '${resizingState.value.header}' to ${newWidth}px`);
};

const stopResize = async () => {
  if (!resizingState.value.isResizing) return;
  
  console.log(`Finish resize: Column '${resizingState.value.header}' set to ${resizingState.value.currentWidth}px`);
  
  // Save final width to database
  resizingState.value.isResizing = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopResize);
  
  // Save to backend
  await saveColumnWidths();
};

// Debounced save function
const saveColumnWidths = useDebounceFn(async () => {
  try {
    console.log(`Saving column widths to backend for ${collectionName.value}`);
    console.log('Widths to save:', collectionSchema.value.ui?.columnWidths);
    
    const response = await fetch(`${API_BASE}/collections/${collectionName.value}/ui-metadata`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        columnWidths: collectionSchema.value.ui?.columnWidths
      })
    });
    
    const result = await response.json();
    console.log('Column width save result:', result);
    
    if (result.success) {
      console.log('Column widths saved successfully');
    } else {
      console.error('Failed to save column widths:', result.error);
      toast({ 
        title: 'Failed to save column widths',
        description: result.error || 'Unknown error',
        variant: 'destructive' 
      });
    }
  } catch (error) {
    console.error('Error saving column widths:', error);
    toast({ 
      title: 'Failed to save column widths', 
      description: String(error),
      variant: 'destructive' 
    });
  }
}, 500);

const resetColumnWidth = async (header: string) => {
  console.log(`Resetting column width for: ${header}`);
  
  const newWidths = { ...columnWidths.value };
  delete newWidths[header];
  
  collectionSchema.value = {
    ...collectionSchema.value,
    ui: {
      ...collectionSchema.value.ui || {},
      columnWidths: newWidths
    }
  };
  
  await saveColumnWidths();
};

const isReferenceField = (field: string) => {
  return getSchemaInfo(field).description?.includes('REF:');
};

const getReferencedCollection = (field: string) => {
  const desc = getSchemaInfo(field).description || '';
  const match = desc.match(/REF:(\w+)/);
  return match ? match[1] : null;
};

const columnWidths = computed(() => {
  return collectionSchema.value?.ui?.columnWidths || {};
});

async function fetchReferenceOptions(collectionName: string) {
  loadingReferences.value[collectionName] = true;
  try {
    // Fetch the referenced collection's schema
    const schemaResponse = await fetch(`${API_BASE}/collections/${collectionName}/schema`);
    const { success: schemaSuccess, data: schemaData, error: schemaError } = await schemaResponse.json();
    
    if (!schemaSuccess) throw new Error(schemaError || 'Failed to fetch schema');
    
    // Determine label field from schema
    let labelField = '_id';
    const properties = schemaData.properties || {};
    
    // Find unique string fields (e.g., username, email, label)
    const uniqueStringFields = Object.keys(properties).filter(field => {
      const prop = properties[field];
      return prop.bsonType === 'string' && prop.unique === true;
    });
    
    // Fallback to common label fields if no unique fields
    if (uniqueStringFields.length > 0) {
      labelField = uniqueStringFields[0];
    } else {
      const candidates = ['label', 'name', 'title', 'username'];
      labelField = candidates.find(f => properties[f]?.bsonType === 'string') || '_id';
    }
    
    // Fetch documents from the referenced collection
    const docsResponse = await fetch(`${API_BASE}/collections/${collectionName}/documents`);
    const { success, data, error } = await docsResponse.json();
    
    if (success) {
      const options = data.map((doc: any) => ({
        id: doc._id.$oid,
        label: doc[labelField] || doc._id.$oid, // Use determined label field
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
      console.log(`Loaded schema for ${newVal}:`, data);
      console.log(`Column widths:`, data.ui?.columnWidths);
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
watch(() => props.name, (newVal) => {
  if (newVal && newVal !== collectionName.value) {
    collectionName.value = newVal;
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
  if (value === undefined || value === null) return '';
  const type = Array.isArray(bsonType) ? bsonType[0] : bsonType;
  
  if (type === 'date' && value instanceof Date) {
    return value.toLocaleString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value, null, 2);
  }
  return String(value);
};

// Function to render a table header cell with resize functionality
const renderTableHeader = (header: string) => {
  const width = resizingState.value.isResizing && resizingState.value.header === header 
    ? resizingState.value.currentWidth 
    : columnWidths.value[header] || 200;

  return {
    key: header,
    width: `${width}px`,
    content: header,
    isRequired: isFieldRequired(header)
  };
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
  <div class="border rounded-md w-full">
    
    <div v-if="errorMessage" class="my-2 p-2 bg-red-100 text-red-700 rounded">
      {{ errorMessage }}
    </div>
    
    <div v-if="isLoading" class="flex justify-center my-8">
      <ReloadIcon class="h-8 w-8 animate-spin text-gray-500" />
    </div>

    <div v-else class="w-full overflow-auto">
      <!-- Use a single consistent table header rendering for all states -->
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead 
              v-for="header in tableHeaders" 
              :key="header" 
              class="border px-4 py-2 bg-gray-100 font-medium relative"
              :style="{ 
                width: resizingState.isResizing && resizingState.header === header 
                  ? `${resizingState.currentWidth}px` 
                  : `${columnWidths[header] || 200}px` 
              }"
            >
              <div class="flex items-center justify-between select-none">
                <span>
                  {{ header }}
                  <span v-if="isFieldRequired(header)" class="text-red-500">*</span>
                </span>
                <div 
                  class="w-1 h-full hover:bg-blue-500 absolute right-0 top-0 cursor-w-resize"
                  :class="[
                    resizingState.isResizing && resizingState.header === header 
                      ? 'bg-blue-500' 
                      : 'bg-gray-200'
                  ]"
                  @mousedown="startResize(header, $event)"
                  @dblclick="resetColumnWidth(header)"
                ></div>
              </div>
            </TableHead>
            <TableHead class="bg-gray-100 font-medium border text-right w-24">
              Actions
            </TableHead>
          </TableRow>
        </TableHeader>
        
        <TableBody>
          <!-- When adding a new document to an empty collection -->
          <TableRow v-if="isAdding && documents.length === 0">
            <TableCell v-for="header in tableHeaders" :key="header" class="border p-2">
              <!-- Timestamp fields - not editable -->
              <span v-if="['created_at', 'updated_at'].includes(header)" class="text-gray-500">
                (auto-generated)
              </span>
              
              <!-- Date field -->
              <Input 
                v-else-if="header !== '_id' && getSchemaInfo(header).bsonType === 'date'"
                v-model="newDocument[header]" 
                type="datetime-local"
                class="w-full h-10"
              />
              
              <!-- Boolean field -->
              <div v-else-if="header !== '_id' && getSchemaInfo(header).bsonType === 'bool'" class="flex items-center">
                <input 
                  type="checkbox" 
                  v-model="newDocument[header]"
                  class="h-4 w-4"
                />
              </div>
              
              <!-- Reference field -->
              <Select 
                v-else-if="header !== '_id' && isReferenceField(header)" 
                v-model="newDocument[header]"
                class="w-full"
              >
                <SelectTrigger>
                  <SelectValue :placeholder="`Select ${getReferencedCollection(header)}`" />
                </SelectTrigger>
                <SelectContent>
                  <ScrollArea class="h-48">
                    <div class="p-1">
                      <Input 
                        v-model="searchQuery[header]"
                        placeholder="Search..."
                        class="mb-2"
                      />
                      <SelectItem 
                        v-for="option in filteredOptions(header)"
                        :key="option.id"
                        :value="option.id"
                      >
                        {{ option.label }}
                      </SelectItem>
                    </div>
                  </ScrollArea>
                </SelectContent>
              </Select>
              
              <!-- Regular field -->
              <Input 
                v-else-if="header !== '_id'" 
                v-model="newDocument[header]" 
                class="w-full h-10"
              />
              
              <!-- ID field (auto) -->
              <span v-else class="text-gray-500">
                (auto)
              </span>
            </TableCell>
            
            <!-- Action buttons -->
            <TableCell class="border p-2">
              <div class="flex space-x-2">
                <Button @click="saveNewDocument" class="bg-black text-white">Save</Button>
                <Button @click="cancelAdding" variant="outline">Cancel</Button>
              </div>
            </TableCell>
          </TableRow>
          
          <!-- Empty state with add document button -->
          <TableRow v-else-if="documents.length === 0" class="h-12">
            <TableCell :colspan="tableHeaders.length + 1" class="text-center p-6 border">
              <div class="flex justify-center items-center">
                <Button @click="startAdding" class="flex items-center gap-2">
                  <span>Add new document</span>
                </Button>
              </div>
            </TableCell>
          </TableRow>

          <!-- Regular Data Rows -->
          <template v-else>
            <TableRow v-for="(doc, rowIndex) in paginatedDocuments" :key="rowIndex">
              <TableCell v-for="header in tableHeaders" :key="`${rowIndex}-${header}`" 
                class="border p-0">
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
                    ></textarea>
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
              <TableCell class="border text-right p-1">
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
          </template>

          <TableRow v-if="isAdding" class="bg-blue-50">
            <TableCell v-for="header in tableHeaders" :key="header" class="p-1">
              <!-- Timestamp fields - not editable -->
              <span v-if="['created_at', 'updated_at'].includes(header)" class="text-gray-500">
                (auto-generated)
              </span>
              
              <!-- Boolean field in inline add mode -->
              <div v-else-if="header !== '_id' && collectionSchema.properties[header]?.bsonType === 'bool'" 
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
      
      <div v-if="totalPages > 1" class="mt-4 p-4">
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
      
      <div class="flex items-center gap-2 p-4">
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

.cursor-col-resize {
  cursor: col-resize;
  user-select: none;
}
</style>