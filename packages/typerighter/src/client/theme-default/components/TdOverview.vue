<script setup lang="ts">
import {
  computed,
} from 'vue';
import {
  contentDisplayName, contentHref,
  type SidebarGroups,
} from '@/shared';

const {
  groups,
} = defineProps<{
  /** Content files grouped by schema */
  groups: SidebarGroups;
}>();

const sortedGroups = computed(() =>
  Object.entries(groups)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([
      schema,
      items,
    ]) => ({
      schema,
      items: [...items].sort((left, right) =>
        contentDisplayName(left).localeCompare(contentDisplayName(right))),
    })));
</script>

<template>
  <div class="py-8">
    <section
      v-for="group in sortedGroups"
      :key="group.schema"
      class="mb-10"
    >
      <div class="border-b-2 border-td-fg pb-2 mb-0.5">
        <h2 class="text-td-h2 font-td-heading leading-td-heading tracking-td-heading m-0 inline">
          {{ group.schema }}
        </h2>
        <span class="text-td-label text-td-gray-600 ml-2">{{ group.items.length }}</span>
      </div>
      <a
        v-for="item in group.items"
        :key="item.path"
        :href="contentHref(item)"
        class="td-index-row"
      >
        <span class="font-td-semibold text-td-body-sm">{{ contentDisplayName(item) }}</span>
        <span class="text-td-label text-td-gray-600 ml-2">{{ item.path }}</span>
      </a>
    </section>
  </div>
</template>

<style scoped>
.td-index-row {
  display: flex;
  align-items: baseline;
  gap: 16px;
  padding: 11px 4px;
  border-bottom: 1px solid var(--color-td-gray-300);
  text-decoration: none;
  color: var(--color-td-fg);
  transition: background-color 0.1s;
}

.td-index-row:hover {
  background: var(--color-td-primary-tint);
}
</style>
