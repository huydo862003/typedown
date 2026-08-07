<script setup lang="ts">
import {
  formatDateString,
  type PropertyDescriptor,
} from '@/shared';

const {
  definition,
  value,
} = defineProps<{
  /** Property descriptor */
  definition: PropertyDescriptor;
  /** Raw value */
  value: unknown;
}>();

function format (val: unknown): string {
  if (val === undefined || val === null) return '';
  if (definition.widget === 'date') return formatDateString(String(val));

  return String(val);
}
</script>

<template>
  {{ format(value) }}
</template>
