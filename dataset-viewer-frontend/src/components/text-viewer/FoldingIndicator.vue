<template>
  <button
    @click="$emit('toggle')"
    class="folding-indicator"
    :class="{
      'folding-indicator--collapsed': isCollapsed,
      'folding-indicator--expanded': !isCollapsed
    }"
    :title="isCollapsed ? '展开代码块' : '折叠代码块'"
  >
    <svg
      class="folding-icon"
      :class="{ 'folding-icon--collapsed': isCollapsed }"
      viewBox="0 0 16 16"
      fill="currentColor"
    >
      <path
        d="M6 2L10 8L6 14"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        fill="none"
      />
    </svg>
  </button>
</template>

<script setup lang="ts">
interface Props {
  isCollapsed: boolean
}

defineProps<Props>()
defineEmits<{
  toggle: []
}>()
</script>

<style scoped>
.folding-indicator {
  @apply inline-flex items-center justify-center w-4 h-4 ml-1 mr-1;
  @apply text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300;
  @apply transition-colors duration-200;
  @apply rounded-sm hover:bg-gray-100 dark:hover:bg-gray-700;
  @apply focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50;
}

.folding-indicator--collapsed {
  @apply text-blue-500 hover:text-blue-600 dark:text-blue-400 dark:hover:text-blue-300;
}

.folding-icon {
  @apply w-3 h-3 transition-transform duration-200;
}

.folding-icon--collapsed {
  @apply transform rotate-0;
}

.folding-icon:not(.folding-icon--collapsed) {
  @apply transform rotate-90;
}
</style>