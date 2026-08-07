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

function format (value_: unknown): string {
  if (value_ === undefined || value_ === null) return '';
  if (definition.widget === 'date') return formatDateString(String(value_));

  return String(value_);
}
</script>

<template>
  {{ format(value) }}
</template>
