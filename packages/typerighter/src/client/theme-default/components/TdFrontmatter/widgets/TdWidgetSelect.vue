<script setup lang="ts">
const {
  value,
} = defineProps<{
  /** Raw value */
  value: unknown;
}>();
</script>

<template>
  <span class="td-widget-pill">{{ value }}</span>
</template>

<style scoped>
.td-widget-pill {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 0.8rem;
  background: var(--color-td-primary-bg-subtle);
  color: var(--color-td-primary-solid);
}
</style>
