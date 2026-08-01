<script setup lang="ts">
import {
  BRAND_VIEWBOX, BRAND_BORDER, BRAND_LETTER, BRAND_ARROW,
} from '@/shared/brand';

const {
  size = 28,
} = defineProps<{
  /** Icon height in pixels */
  size?: number;
}>();
</script>

<template>
  <svg
    fill="none"
    :height="size"
    :width="size * 208 / 128"
    :viewBox="BRAND_VIEWBOX"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path
      clip-rule="evenodd"
      :d="BRAND_BORDER"
      fill-rule="evenodd"
      fill="currentColor"
    />
    <path
      :d="BRAND_LETTER"
      fill="currentColor"
    />
    <path
      :d="BRAND_ARROW"
      fill="currentColor"
    />
  </svg>
</template>
