<script setup lang="ts">
import {
  File, House,
} from 'lucide-vue-next';
import TdTreeNode from './TdTreeNode.vue';
import {
  useRoute,
} from '../../app';
import {
  getTdContentUrl, getTdResourceTitle, INDEX_FILENAME, path,
  type ContentTree,
} from '@/shared';

const {
  tree,
} = defineProps<{
  tree: ContentTree;
}>();

const route = useRoute();
const indexItem = tree.rootItems.find((item) => path.basename(item.filepath, '.td') === INDEX_FILENAME);
const regularRootItems = tree.rootItems.filter((item) => path.basename(item.filepath, '.td') !== INDEX_FILENAME);

function isCurrent (href: string): boolean {
  return route.path === href;
}
</script>

<template>
  <nav>
    <a
      :href="indexItem ? getTdContentUrl(indexItem.filepath) : '/'"
      class="td-root-link"
      :class="{ 'is-active': isCurrent(indexItem ? getTdContentUrl(indexItem.filepath) : '/') }"
    >
      <House
        :size="14"
        class="td-root-link-icon"
      />
      Overview
    </a>
    <a
      v-for="item in regularRootItems"
      :key="item.filepath"
      :href="getTdContentUrl(item.filepath)"
      class="td-root-link"
      :class="{ 'is-active': isCurrent(getTdContentUrl(item.filepath)) }"
    >
      <File
        :size="14"
        class="td-root-link-icon"
      />
      {{ getTdResourceTitle(item.header, item.filepath) }}
    </a>
    <TdTreeNode
      v-for="node in tree.children"
      :key="node.name"
      :node="node"
    />
  </nav>
</template>

<style scoped>
.td-root-link {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 20px;
  font-size: var(--font-size-td-nav);
  color: var(--color-td-gray-700);
  text-decoration: none;
  border-left: 3px solid transparent;
  transition: background-color 0.1s;
}

.td-root-link:hover {
  background-color: var(--color-td-gray-200);
}

.td-root-link.is-active {
  background-color: color-mix(in srgb, var(--color-td-secondary) 20%, transparent);
  border-left-color: var(--color-td-secondary);
  color: var(--color-td-fg);
  font-weight: var(--font-weight-td-semibold);
}

.td-root-link-icon {
  flex-shrink: 0;
  color: var(--color-td-gray-500);
}
</style>
