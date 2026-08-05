<script setup lang="ts">
import {
  computed, ref, watch,
} from 'vue';
import {
  ChevronDown, File, House,
} from 'lucide-vue-next';
import {
  useRoute,
} from '../../app';
import {
  getDirectoryUrl, getTdContentUrl, getTdResourceTitle, INDEX_FILENAME, isUrlAncestorOf, path, unslugify,
  type ContentTreeNode,
} from '@/shared';

const {
  node,
  depth = 0,
  urlPrefix = '',
} = defineProps<{
  node: ContentTreeNode;
  depth?: number;
  urlPrefix?: string;
}>();

const route = useRoute();
const directoryUrl = getDirectoryUrl(urlPrefix, node.name);
const indexItem = node.items.find((item) => path.basename(item.filepath, '.td') === INDEX_FILENAME);
const regularItems = node.items.filter((item) => path.basename(item.filepath, '.td') !== INDEX_FILENAME);
const hasContent = 0 < node.children.length || 0 < node.items.length;

const totalCount = computed(() => countItems(node));

function countItems (n: ContentTreeNode): number {
  return n.items.length + n.children.reduce((sum, child) => sum + countItems(child), 0);
}

const collapsed = ref(!isUrlAncestorOf(directoryUrl, route.path));

watch(() => route.path, (p) => {
  if (isUrlAncestorOf(directoryUrl, p)) collapsed.value = false;
});

function toggle () {
  collapsed.value = !collapsed.value;
}

function isCurrent (href: string): boolean {
  return route.path === href;
}
</script>

<template>
  <div
    v-if="hasContent"
    class="td-tree-node"
  >
    <button
      type="button"
      class="td-tree-label"
      :aria-expanded="!collapsed"
      :style="{
        paddingLeft: `${12 + depth * 12}px`,
      }"
      @click="toggle"
    >
      <ChevronDown
        :size="14"
        class="td-tree-caret"
        :class="{
          'is-collapsed': collapsed,
        }"
      />
      <span class="td-tree-label-text">{{ unslugify(node.name) }}</span>
      <span class="td-tree-count">{{ totalCount }}</span>
    </button>
    <div
      v-if="!collapsed"
      class="td-tree-children"
      :style="{
        marginLeft: `${19 + depth * 12}px`,
      }"
    >
      <a
        :href="indexItem ? getTdContentUrl(indexItem.filepath) : directoryUrl"
        class="td-tree-link"
        :class="{ 'is-active': isCurrent(indexItem ? getTdContentUrl(indexItem.filepath) : directoryUrl) }"
        :style="{
          paddingLeft: `${32 + depth * 12}px`,
        }"
      >
        <House
          :size="14"
          class="td-tree-file-icon"
        />
        {{ unslugify(node.name) }}
      </a>
      <TdTreeNode
        v-for="child in node.children"
        :key="child.name"
        :node="child"
        :depth="depth + 1"
        :url-prefix="directoryUrl"
      />
      <a
        v-for="item in regularItems"
        :key="item.filepath"
        :href="getTdContentUrl(item.filepath)"
        class="td-tree-link"
        :class="{ 'is-active': isCurrent(getTdContentUrl(item.filepath)) }"
        :style="{
          paddingLeft: `${32 + depth * 12}px`,
        }"
      >
        <File
          :size="14"
          class="td-tree-file-icon"
        />
        {{ getTdResourceTitle(item.header, item.filepath) }}
      </a>
    </div>
  </div>
</template>

<style scoped>
.td-tree-label {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  padding: 6px 20px;
  font-size: var(--font-size-td-label);
  letter-spacing: var(--tracking-td-label);
  text-transform: uppercase;
  color: var(--color-td-gray-600);
  background: none;
  border: none;
  cursor: pointer;
}

.td-tree-label:hover {
  color: var(--color-td-fg);
}

.td-tree-label-text {
  flex: 1;
  text-align: left;
}

.td-tree-count {
  font-size: 0.75rem;
  color: var(--color-td-gray-400);
  letter-spacing: normal;
  text-transform: none;
}

.td-tree-caret {
  flex-shrink: 0;
  transition: transform 0.15s;
}

.td-tree-caret.is-collapsed {
  transform: rotate(-90deg);
}

.td-tree-children {
  border-left: 1px solid var(--color-td-gray-300);
}

.td-tree-file-icon {
  flex-shrink: 0;
  color: var(--color-td-gray-500);
}

.td-tree-link {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  font-size: var(--font-size-td-nav);
  color: var(--color-td-gray-700);
  text-decoration: none;
  transition: background-color 0.1s;
}

.td-tree-link:hover {
  background-color: var(--color-td-gray-200);
}

.td-tree-link.is-active {
  background-color: color-mix(in srgb, var(--color-td-secondary) 20%, transparent);
  border-left-color: var(--color-td-secondary);
  color: var(--color-td-fg);
  font-weight: var(--font-weight-td-semibold);
}
</style>
