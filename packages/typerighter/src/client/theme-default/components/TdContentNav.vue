<script setup lang="ts">
import {
  File,
} from 'lucide-vue-next';
import TdTreeNode from './TdTreeNode.vue';
import {
  getTdContentUrl, getTdResourceTitle,
  type ContentTree,
} from '@/shared';

const {
  tree,
} = defineProps<{
  /** Content tree with root items and directory nodes */
  tree: ContentTree;
}>();
</script>

<template>
  <nav>
    <a
      v-for="item in tree.rootItems"
      :key="item.filepath"
      :href="getTdContentUrl(item.filepath)"
      class="td-root-link"
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
  transition: background-color 0.1s;
}

.td-root-link:hover {
  background-color: var(--color-td-gray-200);
}

.td-root-link[aria-current="page"] {
  font-weight: var(--font-weight-td-semibold);
  color: var(--color-td-fg);
}

.td-root-link-icon {
  flex-shrink: 0;
  color: var(--color-td-gray-500);
}
</style>
