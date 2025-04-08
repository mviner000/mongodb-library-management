<!-- src/components/MongoDBDataTable.vue -->
<script setup lang="ts">
import { ref, computed, watch, onMounted, Ref, inject } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { Cross2Icon, ReloadIcon, TrashIcon } from '@radix-icons/vue';
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
const timeoutId = ref<number | null>(null);
  const addingRowError = ref(false);

import { getApiBaseUrl } from '@/utils/api';
import { useDebounceFn } from '@vueuse/core';
import ExcelCellReference from './ExcelCellReference.vue';
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

// Add column letter headers (Excel-like)
const getColumnLabel = (index: number): string => {
  // Convert 0-based index to Excel-like column label (A, B, C, ..., Z, AA, AB, ...)
  let label = '';
  let i = index;
  
  do {
    // Get remainder when divided by 26 (number of letters in alphabet)
    const remainder = i % 26;
    // Convert to corresponding letter (0->A, 1->B, etc.)
    label = String.fromCharCode(65 + remainder) + label;
    // Integer division by 26, minus 1 to account for the previous calculation
    i = Math.floor(i / 26) - 1;
  } while (i >= 0);
  
  return label;
};

// Column letter headers for Excel-like feel
const columnLetters = computed(() => {
  return tableHeaders.value.map((_, index) => getColumnLabel(index));
});

// State for alphabetical header resizing
const alphaResizingState = ref({
  isResizing: false,
  columnIndex: -1,
  startX: 0,
  startWidth: 0,
  currentWidth: 0
});

// Start resize for alphabetical header
const startAlphaResize = (columnIndex: number, event: MouseEvent) => {
  const header = tableHeaders.value[columnIndex];
  console.log(`Starting alpha resize for column ${columnIndex}: ${header}`);
  
  // Get current width from columnWidths or default
  const currentWidth = columnWidths.value[header] || 200;
  
  alphaResizingState.value = {
    isResizing: true,
    columnIndex,
    startX: event.clientX,
    startWidth: currentWidth,
    currentWidth: currentWidth
  };
  
  // Add global event listeners
  document.addEventListener('mousemove', handleAlphaMouseMove);
  document.addEventListener('mouseup', stopAlphaResize);
  
  // Prevent text selection during resize
  event.preventDefault();
};

const handleAlphaMouseMove = (event: MouseEvent) => {
  if (!alphaResizingState.value.isResizing) return;
  
  const delta = event.clientX - alphaResizingState.value.startX;
  const newWidth = Math.max(50, alphaResizingState.value.startWidth + delta);
  
  // Update the current width for visual feedback
  alphaResizingState.value.currentWidth = newWidth;
  
  const header = tableHeaders.value[alphaResizingState.value.columnIndex];
  
  // Update local schema
  const updatedWidths = {
    ...columnWidths.value,
    [header]: newWidth
  };
  
  // Update the UI schema with new widths
  collectionSchema.value = {
    ...collectionSchema.value,
    ui: {
      ...collectionSchema.value.ui || {},
      columnWidths: updatedWidths
    }
  };
  
  console.log(`Alpha resizing column '${header}' to ${newWidth}px`);
};

const stopAlphaResize = async () => {
  if (!alphaResizingState.value.isResizing) return;
  
  const header = tableHeaders.value[alphaResizingState.value.columnIndex];
  console.log(`Finish alpha resize: Column '${header}' set to ${alphaResizingState.value.currentWidth}px`);
  
  // Save final width to database
  alphaResizingState.value.isResizing = false;
  document.removeEventListener('mousemove', handleAlphaMouseMove);
  document.removeEventListener('mouseup', stopAlphaResize);
  
  // Save to backend
  await saveColumnWidths();
};

// Reset width for alphabetical header
const resetAlphaColumnWidth = async (columnIndex: number) => {
  const header = tableHeaders.value[columnIndex];
  console.log(`Resetting alpha column width for: ${header}`);
  
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

// Watch for errorMessage changes
watch(errorMessage, (newVal) => {
  if (newVal) {
    // Clear any existing timeout
    if (timeoutId.value) {
      clearTimeout(timeoutId.value);
    }
    // Set new timeout
    timeoutId.value = setTimeout(() => {
      errorMessage.value = '';
      timeoutId.value = null;
    }, 2500) as unknown as number;
  }
});

// Manual close handler
const closeError = () => {
  if (timeoutId.value) {
    clearTimeout(timeoutId.value);
    timeoutId.value = null;
  }
  errorMessage.value = '';
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

const errorColumn = ref<string | null>(null);
  const errorTimeout = ref<ReturnType<typeof setTimeout> | null>(null);

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
      
      // Check if the error is a duplicate key error
      if (error && error.includes('E11000')) {
        // Extract the field name from the error message
        const match = error.match(/dup key: { (.+?):/);
        if (match && match[1]) {
          const fieldName = match[1];
          console.error(`Error adding new document: Duplicate key error on field '${fieldName}':`, error);
          // Set error column and clear after 5 seconds
          errorColumn.value = fieldName;
          if (errorTimeout.value) clearTimeout(errorTimeout.value);
          errorTimeout.value = setTimeout(() => {
            errorColumn.value = null;
          }, 2500);
        }
      }
      
      addingRowError.value = true;
      setTimeout(() => {
        addingRowError.value = false;
      }, 2500);
    }
  } catch (error) {
    errorMessage.value = `Create failed: ${error}`;
    console.error('Exception while adding new document:', error);
    addingRowError.value = true;
    setTimeout(() => {
      addingRowError.value = false;
    }, 2500);
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

const selectedCell = ref<{ colIndex: number; rowNumber: number } | null>(null);
const handleCellClick = (rowIndex: number, header: string, value: any) => {
  if (['_id', 'created_at', 'updated_at'].includes(header)) return;
  editingCell.value = { rowIndex, header };
  editValue.value = typeof value === 'object' ? JSON.stringify(value, null, 2) : String(value);
  
  // Calculate actual row number and column index
  const actualRowNumber = (currentPage.value - 1) * pageSize.value + rowIndex + 1;
  const colIndex = tableHeaders.value.indexOf(header);
  selectedCell.value = { colIndex, rowNumber: actualRowNumber };
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

const numberColumnWidth = computed(() => {
  // Calculate based on max digits needed (3 digits for up to 999 rows)
  return `${Math.max(3, Math.floor(Math.log10(documents.value.length) + 2))}ch`;
});

const totalTableWidth = computed(() => {
  const dataColumnsWidth = Object.values(columnWidths.value)
    .reduce((acc: number, width: unknown) => acc + Number(width), 0);
  return dataColumnsWidth + 1 + 30 + 60 + 1; // 30px for numbering column, 60px for numbering column, 1px for each border px
});


const isSplit = inject<Ref<boolean>>('isSplit')!; // Inject isSplit from App.vue

// We already have a watch on collectionName that calls fetchDocuments
</script>
<template>
  <MongoDBTableNavbar 
    :isSplitActive="isSplit" 
    class="sticky top-0 z-50" 
  />
  <div class="excel-container w-full">
    
    <div v-if="errorMessage" class="my-2 p-2 bg-red-100 text-red-700 rounded relative pr-8">
      {{ errorMessage }}
      <Button
        @click="closeError"
        variant="ghost"
        size="sm"
        class="absolute right-1 top-1 p-1 h-6 w-6 text-red-700 hover:bg-red-200"
      >
        <Cross2Icon class="h-3 w-3" />
      </Button>
    </div>
    
    <div v-if="isLoading" class="flex justify-center my-8">
      <ReloadIcon class="h-8 w-8 animate-spin text-gray-500" />
    </div>

    <div v-else class="w-full overflow-auto">
      
      <!-- Excel-like table with consistent styling -->
      <ExcelCellReference :selected-cell="selectedCell" />
      <Table class="excel-table" :style="{ width: `${totalTableWidth}px` }">
        
        <!-- Excel-like column headers (A, B, C, ...) -->
        <TableHeader>
         
          <TableRow class="excel-header-row">
            <!-- Row number header -->
            <TableHead 
              class="excel-row-number-header"
              :style="{ 
                width: '30px',
                minWidth: '30px',
                maxWidth: '30px' 
              }"
            >
              <!-- hidden -->
            </TableHead>
            <TableHead 
              v-for="(letter, index) in columnLetters" 
              :key="`letter-${index}`"
              class="excel-column-letter relative"
              :style="{ 
                width: alphaResizingState.isResizing && alphaResizingState.columnIndex === index 
                  ? `${alphaResizingState.currentWidth}px` 
                  : `${columnWidths[tableHeaders[index]] || 200}px` 
              }"
            >
              <div class="flex items-center justify-center">
                <span class="excel-letter">{{ letter }}</span>
                <div 
                  class="excel-resizer absolute right-0 top-0"
                  :class="[
                    alphaResizingState.isResizing && alphaResizingState.columnIndex === index 
                      ? 'excel-resizer-active' 
                      : ''
                  ]"
                  @mousedown="startAlphaResize(index, $event)"
                  @dblclick="resetAlphaColumnWidth(index)"
                ></div>
              </div>
            </TableHead>
            <!-- Actions column in alpha header row -->
            <TableHead class="excel-column-letter excel-actions-header w-24">
              <!-- Empty for alignment -->
            </TableHead>
          </TableRow>
        </TableHeader>
        
        <TableHeader>
          <TableRow>
            <!-- Row number column header -->
            <TableHead 
              class="excel-row-number-header"
              :style="{ 
                width: numberColumnWidth,
                minWidth: numberColumnWidth,
                maxWidth: numberColumnWidth 
              }"
            >
              <!-- hidden -->
            </TableHead>
            <TableHead 
              v-for="header in tableHeaders" 
              :key="header" 
              class="excel-column-header relative"
              :class="{ 'error-column-header': header === errorColumn }"
              :style="{ 
                width: resizingState.isResizing && resizingState.header === header 
                  ? `${resizingState.currentWidth}px` 
                  : `${columnWidths[header] || 200}px` 
              }"
            >
              <div class="flex items-center justify-between">
                <span>
                  {{ header }}
                  <span v-if="isFieldRequired(header)" class="text-red-500">*</span>
                </span>
                <div 
                  class="excel-resizer absolute right-0 top-0"
                  :class="[
                    resizingState.isResizing && resizingState.header === header 
                      ? 'excel-resizer-active' 
                      : ''
                  ]"
                  @mousedown="startResize(header, $event)"
                  @dblclick="resetColumnWidth(header)"
                ></div>
              </div>
            </TableHead>
            <TableHead 
              class="excel-column-header excel-actions-header" 
              :style="{ width: '30px' }"
            >
              Actions
            </TableHead>
          </TableRow>
        </TableHeader>
        
        <TableBody>
          <!-- Regular Data Rows -->
          <template v-if="documents.length > 0">
            <TableRow 
              v-for="(doc, rowIndex) in paginatedDocuments" 
              :key="rowIndex"
              class="excel-data-row"
              :class="{'excel-row-even': rowIndex % 2 === 0}"
            >
              <!-- Row number -->
              <TableCell 
                class="excel-row-number"
                :style="{ 
                  width: numberColumnWidth,
                  minWidth: numberColumnWidth,
                  maxWidth: numberColumnWidth 
                }"
              >
                {{ (currentPage - 1) * pageSize + rowIndex + 1 }}
              </TableCell>
              <TableCell 
                v-for="header in tableHeaders" 
                :key="`${rowIndex}-${header}`" 
                class="excel-cell"
                :class="{
                  'error-column-cell': header === errorColumn,
                  'excel-cell-selected': editingCell?.rowIndex === rowIndex && editingCell?.header === header
                  }"
              >
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
                        class="excel-checkbox"
                      />
                    </div>
                    <!-- Reference field editing -->
                    <div v-else-if="isReferenceField(header)" class="p-1">
                      <Select v-model="editValue" @update:modelValue="saveEdit" class="excel-select">
                        <SelectTrigger>
                          <SelectValue :placeholder="`Select ${getReferencedCollection(header)}`" :model-value="editValue" />
                        </SelectTrigger>
                        <SelectContent>
                          <ScrollArea class="h-48">
                            <div class="p-1">
                              <Input 
                                v-model="searchQuery[header]"
                                placeholder="Search..."
                                class="mb-2 excel-input"
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
                      class="excel-input excel-date-input"
                    />
                    <!-- Default textarea for other fields -->
                    <textarea v-else
                      v-model="editValue"
                      @blur="saveEdit"
                      @keyup.ctrl.enter="saveEdit"
                      class="excel-textarea"
                      rows="1"
                    ></textarea>
                  </div>
                  <div v-else
                    class="excel-cell-content"
                    :class="{
                      'excel-cell-editable': !['created_at', 'updated_at'].includes(header),
                      'excel-cell-readonly': ['created_at', 'updated_at'].includes(header)
                    }"
                    @click="!['created_at', 'updated_at', '_id'].includes(header) && handleCellClick(rowIndex, header, doc[header])"
                  >
                    <!-- Show boolean values with checkboxes in read-only mode -->
                    <div v-if="collectionSchema.properties[header]?.bsonType === 'bool'" class="flex justify-center">
                      <input type="checkbox" :checked="doc[header]" disabled class="excel-checkbox" />
                    </div>
                    <!-- Show reference field labels in read-only mode -->
                    <div v-else-if="isReferenceField(header)" class="excel-reference-value">
                      {{ getReferenceLabel(header, doc[header]) || doc[header] }}
                    </div>
                    <!-- Add disabled display for timestamp fields -->
                    <template v-else-if="['created_at', 'updated_at'].includes(header)">
                      <span class="excel-timestamp">
                        {{ formatSchemaValue(doc[header], collectionSchema.properties[header]?.bsonType) }}
                      </span>
                    </template>
                    <template v-else>
                      {{ formatSchemaValue(doc[header], collectionSchema.properties[header]?.bsonType) }}
                    </template>
                  </div>
                </div>
              </TableCell>
              <TableCell class="excel-cell excel-actions-cell">
                <Button
                  variant="ghost"
                  size="sm"
                  class="excel-delete-button"
                  @click="deleteDocument(doc._id.$oid)"
                >
                  <TrashIcon class="h-4 w-4" />
                </Button>
              </TableCell>
            </TableRow>
          </template>

          <!-- Add new document form row -->
          <TableRow v-if="isAdding" 
            class="excel-new-row"
            :class="['excel-new-row', { 'excel-new-row-error': addingRowError }]"
          >
            <!-- Row number for new row -->
            <TableCell class="excel-row-number">
              {{ documents.length + 1 }}
            </TableCell>
            <TableCell v-for="header in tableHeaders" :key="header" class="excel-cell">
              <!-- Timestamp fields - not editable -->
              <span v-if="['created_at', 'updated_at'].includes(header)" class="excel-timestamp">
                (auto-generated)
              </span>
              
              <!-- Boolean field in inline add mode -->
              <div v-else-if="header !== '_id' && collectionSchema.properties[header]?.bsonType === 'bool'" 
                class="flex items-center justify-center">
                <input 
                  type="checkbox" 
                  v-model="newDocument[header]"
                  class="excel-checkbox"
                />
              </div>
              <!-- Reference field in inline add mode -->
              <div v-else-if="header !== '_id' && isReferenceField(header)" class="h-8">
                <Select v-model="newDocument[header]" class="excel-select">
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
                        class="mb-1 mx-1 excel-input"
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
                class="excel-input"
              />
              <span v-else class="excel-auto-id">(auto)</span>
            </TableCell>
            <!-- the last table cell intentionally blanked -->
          </TableRow>

          <!-- Add new document button row -->
          <TableRow 
            v-if="!isAdding" 
            class="excel-add-row" 
            @click="startAdding"
          >
            <TableCell :colspan="tableHeaders.length + 2" class="excel-add-cell">
              <div class="inline-flex items-center gap-2 excel-add-button">
                <PlusCircledIcon class="h-4 w-4" />
                <span class="text-sm">
                  {{ documents.length === 0 ? 'Add first document' : 'Add new document' }}
                </span>
              </div>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>

  <!-- Floating Add Widget -->
  <div
    v-if="isAdding"
    class="sticky top-2 left-2 z-20 p-3 shadow-lg flex items-center space-x-2 w-auto rounded-md bg-green-50"
  >
    <Button @click="saveNewDocument" size="sm" class="bg-green-600 hover:bg-green-700 text-white">
      <PlusCircledIcon class="w-4 h-4" />
      Save
    </Button>
    <Button @click="cancelAdding" variant="outline" size="sm" class="border-green-600 text-green-700 hover:bg-green-100">
      <Cross2Icon class="w-4 h-4" />
      Cancel
    </Button>
  </div>
      
      <div v-if="totalPages > 1" class="excel-pagination">
        <Pagination :page="currentPage" :itemsPerPage="pageSize" :total="documents.length"
          @update:page="onPageChange" :siblingCount="1">
          <PaginationList>
            <PaginationListItem :value="1">
              <PaginationFirst :disabled="currentPage === 1" @click="currentPage = 1" class="excel-pagination-button" />
            </PaginationListItem>
            <PaginationListItem :value="Math.max(1, currentPage - 1)">
              <PaginationPrev :disabled="currentPage === 1" 
                @click="currentPage = Math.max(1, currentPage - 1)" class="excel-pagination-button" />
            </PaginationListItem>
            <PaginationListItem :value="Math.min(totalPages, currentPage + 1)">
              <PaginationNext :disabled="currentPage === totalPages" 
                @click="currentPage = Math.min(totalPages, currentPage + 1)" class="excel-pagination-button" />
            </PaginationListItem>
            <PaginationListItem :value="totalPages">
              <PaginationLast :disabled="currentPage === totalPages" 
                @click="currentPage = totalPages" class="excel-pagination-button" />
            </PaginationListItem>
          </PaginationList>
        </Pagination>
      </div>
      
      <div v-if="!isAdding" class="excel-footer">
        <span class="excel-page-size-label">Rows per page:</span>
        <Select v-model="pageSize" class="excel-page-size-select">
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
        
        <span class="excel-status-info">
          Showing {{ (currentPage - 1) * pageSize + 1 }} to 
          {{ Math.min(currentPage * pageSize, documents.length) }} of 
          {{ documents.length }} entries
        </span>
      </div>
    </div>
  </div>
</template>

<style>
/* Excel-inspired container */
.excel-container {
  font-family: 'Segoe UI', Arial, sans-serif;
  border: 1px solid #d4d4d8;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
  border-radius: 2px;
  background-color: #ffffff;
}

/* Excel table styling */
.excel-table {
  table-layout: fixed;
  min-width: fit-content;
}

/* Excel header row */
.excel-header-row {
  background-color: #f3f3f3;
}

/* Excel column headers */
.excel-column-header {
  background-color: #f3f3f3;
  border: 1px solid #d0d0d0;
  padding: 6px 8px;
  font-weight: 600;
  font-size: 14px;
  color: #212121;
  position: relative;
  text-align: left;
}

/* Excel column letter headers (A, B, C) */
.excel-column-letter {
  background-color: #e6e6e6;
  border: 1px solid #d0d0d0;
  padding: 4px 8px;
  font-weight: 600;
  font-size: 14px;
  color: #616161;
  text-align: center;
}

.excel-letter {
  font-weight: 700;
}

/* Excel row number header */
.excel-row-number-header {
  background-color: #e6e6e6;
  border: 1px solid #d0d0d0;
  padding: 6px 4px;
  font-weight: 600;
  font-size: 14px;
  color: #616161;
  text-align: center;
}

/* Excel row number cells */
.excel-row-number {
  background-color: #f3f3f3;
  border: 1px solid #d0d0d0;
  padding: 4px;
  font-weight: 500;
  font-size: 14px;
  color: #616161;
  text-align: center;
}

/* Excel actions header */
.excel-actions-header {
  background-color: #f3f3f3;
  border: 1px solid #d0d0d0;
  text-align: center;
}

/* Excel data rows */
.excel-data-row:hover {
  background-color: #edf5fd;
}

/* Excel data cell */
.excel-cell {
  border: 1px solid #d0d0d0;
  padding: 0;
  font-size: 14px;
  color: #212121;
  position: relative;
}

/* Excel cell content */
.excel-cell-content {
  padding: 6px 8px;
  min-height: 40px;
}

/* Excel cell editable */
.excel-cell-editable {
  cursor: pointer;
}

.excel-cell-editable:hover {
  background-color: #e8f3fd;
}

/* Excel cell readonly */
.excel-cell-readonly {
  cursor: not-allowed;
  opacity: 0.8;
  background-color: #f9f9f9;
}

/* Excel cell selected - active cell styling */
.excel-cell-selected {
  outline: 2px solid #217346;
  outline-offset: -2px;
  position: relative;
  z-index: 1;
}

/* Excel textarea */
.excel-textarea {
  width: 100%;
  height: 100%;
  padding: 6px 8px;
  font-family: 'Segoe UI', Arial, sans-serif;
  font-size: 14px;
  border: none;
  resize: none; /* Change from vertical to none */
  min-height: 40px; /* Reduce from 80px to 40px to match other row heights */
  outline: none;
  box-shadow: none;
  overflow: hidden; /* Hide overflow */
}

.excel-new-row .excel-cell {
  overflow: visible; /* Ensure content is visible */
  height: 40px; /* Set a consistent height */
}

/* Ensure inputs in the new row don't cause scrolling */
.excel-new-row .excel-input {
  overflow: hidden;
}

.excel-textarea:focus {
  outline: none;
  box-shadow: none;
}

/* Excel input */
.excel-input {
  height: 100%;
  min-height: 32px;
  border-radius: 0;
  border: none;
  box-shadow: none;
  font-size: 14px;
  padding: 4px 6px;
}

.excel-input:focus-visible {
  outline: none;
  box-shadow: none;
  border: none;
  ring: none;
}

/* Excel date input */
.excel-date-input {
  padding: 2px 4px;
  font-size: 14px;
}

/* Excel checkbox */
.excel-checkbox {
  height: 16px;
  width: 16px;
  cursor: pointer;
  accent-color: #217346;
}

/* Excel reference value */
.excel-reference-value {
  color: #0066cc;
  cursor: pointer;
}

/* Excel timestamp value */
.excel-timestamp {
  color: #666666;
  font-style: italic;
  font-size: 14px;
}

/* Excel auto ID */
.excel-auto-id {
  color: #888888;
  font-style: italic;
  padding: 0 8px;
  font-size: 14px;
}

/* Excel actions cell */
.excel-row-number-header,
.excel-row-number {
  width: 30px !important;
  min-width: 30px !important;
  max-width: 30px !important;
  border-left: 1px solid #d0d0d0;
}

.excel-actions-header,
.excel-actions-cell {
  width: 60px !important;
  min-width: 60px !important;
  max-width: 60px !important;
  border-right: 1px solid #d0d0d0;
}

/* Excel delete button */
.excel-delete-button {
  color: #d32f2f;
}

.excel-delete-button:hover {
  color: #b71c1c;
  background-color: #ffebee;
}

/* Excel new row */
.excel-new-row {
  background-color: #e8f5e9;
}

/* Excel save button */
.excel-save-button {
  background-color: #217346;
  color: white;
  font-size: 14px;
  height: 28px;
}

.excel-save-button:hover {
  background-color: #1e6b41;
}

/* Excel cancel button */
.excel-cancel-button {
  color: #666666;
  font-size: 14px;
  height: 28px;
}

/* Excel add row */
.excel-add-row {
  cursor: pointer;
}

.excel-add-row:hover {
  background-color: #f0f8ff;
}

/* Excel add cell */
.excel-add-cell {
  text-align: center;
  padding: 8px 0;
}

/* Excel add button */
.excel-add-button {
  color: #217346;
  font-weight: 500;
}

/* Excel column resizer */
.excel-resizer {
  width: 5px;
  height: 100%;
  cursor: col-resize;
  position: absolute;
  right: 0;
  top: 0;
  background-color: transparent;
}

.excel-resizer:hover {
  background-color: #217346;
}

.excel-resizer-active {
  background-color: #217346;
}

/* Excel select */
.excel-select {
  font-size: 14px;
}

/* Excel row alternate colors */
.excel-row-even {
  background-color: #f9f9f9;
}

/* Excel pagination */
.excel-pagination {
  margin-top: 16px;
  padding: 8px;
  border-top: 1px solid #e0e0e0;
  display: flex;
  justify-content: center;
}

.excel-pagination-button {
  color: #217346;
}

/* Excel footer */
.excel-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-top: 1px solid #e0e0e0;
  background-color: #f9f9f9;
}

.excel-page-size-label {
  font-size: 14px;
  color: #666666;
}

.excel-page-size-select {
  font-size: 14px;
}

.excel-status-info {
  margin-left: 16px;
  font-size: 14px;
  color: #666666;
}

.error-column-header {
  background-color: #fee2e2 !important;
  border: 2px solid #ef4444 !important;
}

.error-column-cell {
  background-color: #fef2f2 !important;
  border-right: 2px solid #ef4444 !important;
  border-left: 2px solid #ef4444 !important;
  animation: error-flash 5s;
}

.excel-new-row-error {
  background-color: #fee2e2 !important;
  animation: error-flash 5s;
}

@keyframes error-flash {
  0% { background-color: #fee2e2; }
  90% { background-color: #fef2f2; }
  100% { background-color: inherit; }
}
</style>