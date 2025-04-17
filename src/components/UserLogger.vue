<template>
  <button
    v-if="showButton"
    @click="refetchUser"
    :disabled="userStore.loading"
    class="refresh-button"
  >
    {{ userStore.loading ? 'Loading...' : 'Refresh User' }}
  </button>
</template>

<script setup lang="ts">
  import { useUserStore } from '@/store/useUserStore'
  import { onMounted } from 'vue'

  const props = withDefaults(
    defineProps<{
      showButton?: boolean
    }>(),
    {
      showButton: true,
    }
  )

  // Access user store
  const userStore = useUserStore()

  /**
   * Log the current user ID
   */
  const logUserId = () => {
    const userId = userStore.user?.id

    if (userId) {
      console.log(`User ID: ${userId}`)
    } else {
      console.log('No user ID available')
    }
  }

  /**
   * Refetch user data from the API
   */
  const refetchUser = async () => {
    console.log('Refetching user data...')
    await userStore.fetchUser()
    logUserId()
  }

  // Log user ID on mount
  onMounted(() => {
    logUserId()
  })

  // Expose methods for external use
  defineExpose({
    logUserId,
    refetchUser,
  })
</script>

<style scoped>
  .refresh-button {
    padding: 6px 12px;
    background-color: #4a7dfc;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .refresh-button:disabled {
    background-color: #9db5f2;
    cursor: not-allowed;
  }
</style>
