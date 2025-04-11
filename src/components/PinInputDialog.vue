<!-- src/components/PinInputDialog.vue -->

<template>
  <Dialog :open="visible" @update:open="handleDialogChange">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>{{ title }}</DialogTitle>
      </DialogHeader>
      <div>
        <div class="-ml-[30px] flex justify-center gap-2 relative">
          <input
            v-for="(_, index) in pinValues"
            :key="index"
            :ref="el => { if (el) pinInputRefs[index] = el as HTMLInputElement }"
            v-model="pinValues[index]"
            :type="showPin ? 'text' : 'password'"
            inputmode="numeric"
            pattern="[0-9]*"
            maxlength="1"
            class="h-10 w-10 rounded-md border border-input text-center text-base focus:outline-none focus:ring-2 focus:ring-blue-500"
            @input="event => handlePinInput(event, index)"
            @keydown="event => handleKeyDown(event, index)"
            :aria-label="`PIN digit ${index + 1}`"
          />
          <button 
            type="button" 
            class="absolute top-[9px] right-7 translate-x-full flex items-center text-gray-500 hover:text-gray-700"
            @click="togglePinVisibility"
            aria-label="Toggle PIN visibility"
          >
            <!-- Eye icon for show -->
            <svg v-if="!showPin" xmlns="http://www.w3.org/2000/svg" class="h-[22px] w-[22px]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
            </svg>
            <!-- Eye-off icon for hide -->
            <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-[22px] w-[22px]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
            </svg>
          </button>
        </div>
      </div>
      
      <div class="mt-4 flex justify-between gap-2">
        <div class="text-xs text-left flex items-center flex-grow">
          <span v-if="errorMessage" class="flex items-center" :class="errorTypeClass">
            <!-- Warning triangle icon for format errors -->
            <svg v-if="errorType === 'format'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <!-- X-circle icon for invalid PIN -->
            <svg v-else-if="errorType === 'invalid'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m-6 4h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            {{ errorMessage }}
          </span>
        </div>
        <DialogClose class="px-4 py-2 text-gray-600 hover:text-gray-800 rounded-md flex-shrink-0" @click="handleCancel">
          Cancel
        </DialogClose>
      </div>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed } from 'vue';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogClose } from '@/components/ui/dialog';

const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
  correctPin: {
    type: String,
    default: '123456',
  },
  title: {
    type: String,
    default: 'Enter PIN',
  },
  pinLength: {
    type: Number,
    default: 6,
  },
  initiallyShowPin: {
    type: Boolean,
    default: false,
  }
});

const emit = defineEmits(['update:visible', 'correct', 'incorrect', 'cancel']);

const pinValues = ref<string[]>(Array(props.pinLength).fill(''));
const errorMessage = ref('');
const errorType = ref<'format' | 'invalid' | null>(null);
const pinInputRefs = ref<(HTMLInputElement | null)[]>(Array(props.pinLength).fill(null));
const attemptCount = ref(0);
const showPin = ref(props.initiallyShowPin);

// Handle dialog open/close
const handleDialogChange = (isOpen: boolean) => {
  if (!isOpen) {
    handleCancel();
  }
  emit('update:visible', isOpen);
};

// Computed property for error message color based on error type
const errorTypeClass = computed(() => {
  if (errorType.value === 'format') {
    return 'text-yellow-600'; // Yellow for "Numbers only (0-9)"
  } else if (errorType.value === 'invalid') {
    return 'text-red-600'; // Red for "Invalid PIN code"
  }
  return '';
});

watch(() => props.pinLength, (newLength) => {
  if (pinValues.value.length !== newLength) {
    pinValues.value = Array(newLength).fill('');
    pinInputRefs.value = Array(newLength).fill(null);
  }
}, { immediate: true }); // Use immediate to set initial arrays based on prop

// Watch for initiallyShowPin changes
watch(() => props.initiallyShowPin, (newValue) => {
  showPin.value = newValue;
});

// Watch for visibility changes
watch(() => props.visible, (isVisible) => {
  if (isVisible) {
    // Reset state and clear inputs every time it opens
    resetState(true);
    nextTick(() => {
      focusInput(0); // Focus first input when dialog opens
    });
  }
});

const enteredPin = computed(() => pinValues.value.join(''));

// --- Methods ---

const togglePinVisibility = () => {
  showPin.value = !showPin.value;
};

const clearInputsOnly = () => {
  for (let i = 0; i < pinValues.value.length; i++) {
    pinValues.value[i] = '';
  }
};

const resetState = (clearInputs: boolean = true) => {
  if (clearInputs) {
    clearInputsOnly();
  }
  // Always clear errors when resetting state
  errorMessage.value = '';
  errorType.value = null;
  attemptCount.value = 0; // Reset attempts when dialog is reopened
  // Reset PIN visibility to initial value
  showPin.value = props.initiallyShowPin;
};

const focusInput = (index: number) => {
  nextTick(() => {
    const input = pinInputRefs.value[index];
    if (input) {
      input.focus();
      input.select();
    }
  });
};

const handlePinInput = (event: Event, index: number) => {
  const input = event.target as HTMLInputElement;
  const value = input.value;

  // Only clear format error messages, keep the invalid PIN error visible
  if (errorType.value === 'format') {
    errorMessage.value = '';
    errorType.value = null;
  }

  if (!/^\d?$/.test(value)) {
    pinValues.value[index] = '';
    // Show specific message for non-digit input
    errorMessage.value = 'Numbers only (0-9)';
    errorType.value = 'format';
    // Keep focus on the current input for correction
    nextTick(() => input.select());
    return;
  }

  // If the value is a digit (or empty string after deletion)
  pinValues.value[index] = value;

  if (value && index < props.pinLength - 1) {
    focusInput(index + 1);
  }

  // Check if PIN is complete (all boxes filled)
  const isComplete = pinValues.value.every(val => val !== '');
  if (isComplete) {
    checkPin();
  }
};

const handleKeyDown = (event: KeyboardEvent, index: number) => {
  if (event.key === 'Backspace') {
    // If current input is empty, move focus back
    if (!pinValues.value[index] && index > 0) {
      focusInput(index - 1);
      event.preventDefault();
    }
    // Let default backspace clear the current input if it has value
  }

  if (event.key === 'ArrowLeft' && index > 0) {
    focusInput(index - 1);
    event.preventDefault();
  }
  if (event.key === 'ArrowRight' && index < props.pinLength - 1) {
    focusInput(index + 1);
    event.preventDefault();
  }
};

// Check PIN code
const checkPin = () => {
  if (enteredPin.value === props.correctPin) {
    errorMessage.value = '';
    errorType.value = null;
    attemptCount.value = 0; // Reset count on success
    emit('correct', enteredPin.value);
  } else {
    // Increment attempt counter
    attemptCount.value++;
    
    // Clear inputs on invalid PIN to allow easy re-entry
    clearInputsOnly();
    
    // Show error message with attempt count
    errorMessage.value = `Invalid PIN code. Attempt ${attemptCount.value}`;
    errorType.value = 'invalid';
    emit('incorrect', attemptCount.value);
    
    // Focus the first input after clearing for a fresh start
    focusInput(0);
  }
};

const handleCancel = () => {
  resetState(true); // Ensure inputs are cleared on cancel
  emit('cancel');
  emit('update:visible', false);
};
</script>

<style scoped>
/* Add any component-specific styles here if needed */
input[type="text"],
input[type="password"] {
  appearance: textfield;
  -moz-appearance: textfield; /* Firefox */
  -webkit-appearance: textfield; /* Safari and older Chrome */
}

input::-webkit-outer-spin-button,
input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
</style>