<script setup lang="ts">
const {
  label,
} = defineProps<{
  /** Accessible label */
  label: string;
}>();
</script>

<template>
  <button
    type="button"
    class="inline-flex items-center justify-center cursor-pointer"
    :title="label"
    :aria-label="label"
  >
    <slot />
  </button>
</template>
