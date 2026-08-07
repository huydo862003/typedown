<script setup lang="ts">
const {
  value,
} = defineProps<{
  /** Raw value */
  value: unknown;
}>();
</script>

<template>
  <span class="td-widget-checkbox">{{ value ? '☑' : '☐' }}</span>
</template>

<style scoped>
.td-widget-checkbox {
  font-size: 1rem;
}
</style>
