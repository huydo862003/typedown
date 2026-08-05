<script setup lang="ts">
import {
  ref, watch,
} from 'vue';
import {
  ChevronDown, File,
} from 'lucide-vue-next';
import {
  useRoute,
} from '../../app';
import {
  getDirectoryUrl, getTdContentUrl, getTdResourceTitle, isUrlAncestorOf, unslugify,
  type ContentTreeNode,
} from '@/shared';

const {
  node,
  depth = 0,
  urlPrefix = '',
} = defineProps<{
  /** Tree node to render */
  node: ContentTreeNode;
  /** Nesting depth for indentation */
  depth?: number;
  /** Accumulated path prefix for building directory hrefs */
  urlPrefix?: string;
}>();

const route = useRoute();
const directoryUrl = getDirectoryUrl(urlPrefix, node.name);
const hasContent = 0 < node.children.length || 0 < node.items.length;

// Auto-expand if the current route falls within this directory subtree
const collapsed = ref(!isUrlAncestorOf(directoryUrl, route.path));

watch(() => route.path, (path) => {
  if (isUrlAncestorOf(directoryUrl, path)) collapsed.value = false;
});

function toggle () {
  collapsed.value = !collapsed.value;
}
</script>

<template>
  <div
    v-if="hasContent"
    class="td-tree-node"
  >
    <div
      class="td-tree-label"
      :style="{
        paddingLeft: `${12 + depth * 12}px`,
      }"
    >
      <button
        type="button"
        class="td-tree-toggle"
        :aria-expanded="!collapsed"
        aria-label="Toggle section"
        @click="toggle"
      >
        <ChevronDown
          :size="14"
          class="td-tree-caret"
          :class="{
            'is-collapsed': collapsed,
          }"
        />
      </button>
      <a
        :href="directoryUrl"
        class="td-tree-label-link"
      >
        {{ unslugify(node.name) }}
      </a>
    </div>
    <div v-if="!collapsed">
      <TdTreeNode
        v-for="child in node.children"
        :key="child.name"
        :node="child"
        :depth="depth + 1"
        :url-prefix="directoryUrl"
      />
      <a
        v-for="item in node.items"
        :key="item.filepath"
        :href="getTdContentUrl(item.filepath)"
        class="td-tree-link"
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
  gap: 0;
  width: 100%;
  padding: 6px 20px;
  font-size: var(--font-size-td-label);
  letter-spacing: var(--tracking-td-label);
  text-transform: uppercase;
  color: var(--color-td-gray-600);
}

.td-tree-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px;
  color: inherit;
  flex-shrink: 0;
}

.td-tree-label-link {
  text-decoration: none;
  color: inherit;
  transition: color 0.15s;
}

.td-tree-label-link:hover {
  color: var(--color-td-fg);
}

.td-tree-caret {
  flex-shrink: 0;
  transition: transform 0.15s;
}

.td-tree-caret.is-collapsed {
  transform: rotate(-90deg);
}

.td-tree-file-icon {
  flex-shrink: 0;
  color: var(--color-td-gray-500);
}

.td-tree-link {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 20px 5px 32px;
  font-size: var(--font-size-td-nav);
  color: var(--color-td-gray-700);
  text-decoration: none;
  border-left: 3px solid transparent;
  transition: background-color 0.1s;
}

.td-tree-link:hover {
  background-color: var(--color-td-gray-200);
}

.td-tree-link[aria-current="page"] {
  border-left-color: var(--color-td-primary);
  font-weight: var(--font-weight-td-semibold);
  color: var(--color-td-fg);
}
</style>
