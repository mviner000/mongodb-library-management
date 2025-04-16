<template>
  <div class="sidebar-container">
    <!-- Sidebar content -->
    <transition name="slide">
      <div
        v-if="isOpen"
        class="sidebar"
      >
        <div class="sidebar-content">
          <h2 class="sidebar-title">📌 Pinned Items</h2>
          Pin items go here
        </div>
      </div>
    </transition>

    <!-- Half-circle trigger button - Excel style -->
    <div
      class="trigger-button opacity-50 hover:opacity-100"
      :class="{ open: isOpen }"
      @click="toggleSidebar"
    >
      <div class="icon-container">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line
            x1="3"
            y1="12"
            x2="21"
            y2="12"
          ></line>
          <line
            v-if="!isOpen"
            x1="3"
            y1="6"
            x2="21"
            y2="6"
          ></line>
          <line
            v-if="!isOpen"
            x1="3"
            y1="18"
            x2="21"
            y2="18"
          ></line>
          <line
            v-if="isOpen"
            x1="18"
            y1="6"
            x2="6"
            y2="18"
          ></line>
          <line
            v-if="isOpen"
            x1="6"
            y1="6"
            x2="18"
            y2="18"
          ></line>
        </svg>
      </div>
    </div>
  </div>
</template>

<script>
  export default {
    name: 'StickyLeftSidebar',
    props: {
      isOpen: {
        type: Boolean,
        default: true,
      },
    },
    methods: {
      toggleSidebar() {
        this.$emit('toggle')
      },
    },
  }
</script>

<style scoped>
  .sidebar-container {
    position: relative;
    z-index: 40;
  }

  .sidebar {
    position: fixed;
    top: 0;
    left: 0;
    height: 100vh;
    width: 280px;
    background-color: #f8f9fa;
    border-right: 1px solid #d4d4d8;
    z-index: 40;
    margin-top: 0px; /* Adjust based on your Excel header height */
  }

  .sidebar-content {
    padding: 16px;
  }

  .sidebar-title {
    font-size: 14px;
    font-weight: 500;
    color: #4b5563;
    margin-bottom: 16px;
  }

  .sidebar-menu {
    display: flex;
    flex-direction: column;
  }

  .trigger-button {
    position: fixed;
    top: 75%;
    right: auto;
    left: 0;
    transform: translateY(-50%);
    width: 22px;
    height: 60px;
    background-color: #217346; /* Excel green color */
    border-radius: 0 4px 4px 0;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      all 0.3s ease,
      opacity 0.2s ease;
    z-index: 41;
    box-shadow: 1px 1px 3px rgba(0, 0, 0, 0.1);
  }

  .trigger-button.open {
    left: 280px;
    background-color: #185a34; /* Darker green when open */
  }

  .icon-container {
    display: flex;
    justify-content: center;
    align-items: center;
    color: white;
    margin-left: -2px;
  }

  /* Animation */
  .slide-enter-active,
  .slide-leave-active {
    transition: transform 0.3s ease;
  }

  .slide-enter-from,
  .slide-leave-to {
    transform: translateX(-100%);
  }
</style>
