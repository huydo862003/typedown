<script setup lang="ts">
import {
  extractRef,
} from './ref';

const {
  value,
} = defineProps<{
  /** Raw value ({ $ref: { url, name } }) */
  value: unknown;
}>();

const resolved = extractRef(value);
</script>

<template>
  <a
    v-if="resolved"
    :href="resolved.url"
    class="td-widget-ref"
  >{{ resolved.name }}</a>
  <span v-else>{{ value }}</span>
</template>

<style scoped>
.td-widget-ref {
  color: var(--color-td-link);
  text-decoration: none;
}

.td-widget-ref:hover {
  color: var(--color-td-link-hover);
  text-decoration: underline;
}
</style>
