<script setup lang="ts">
import {
  ref,
} from 'vue';
import {
  ChevronDown,
} from 'lucide-vue-next';
import {
  contentDisplayName, contentHref,
  type SidebarGroups,
} from '@/shared';

const {
  groups,
} = defineProps<{
  /** Content grouped by schema */
  groups: SidebarGroups;
}>();

const collapsed = ref<Record<string, boolean>>({});

function toggleGroup (schema: string): void {
  collapsed.value[schema] = !collapsed.value[schema];
}
</script>

<template>
  <nav>
    <div
      v-for="(items, schema) in groups"
      :key="schema"
      class="td-content-nav-group"
    >
      <button
        type="button"
        class="td-content-nav-label"
        :aria-expanded="!collapsed[schema]"
        @click="() => toggleGroup(String(schema))"
      >
        <ChevronDown
          :size="14"
          class="td-content-nav-caret"
          :class="{
            'is-collapsed': collapsed[schema],
          }"
        />
        {{ schema }}
      </button>
      <div v-if="!collapsed[schema]">
        <a
          v-for="item in items"
          :key="item.path"
          :href="contentHref(item)"
          class="td-content-nav-link"
        >
          {{ contentDisplayName(item) }}
        </a>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.td-content-nav-group {
  margin-bottom: 4px;
}

.td-content-nav-label {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 6px 20px;
  font-size: var(--font-size-td-label);
  letter-spacing: var(--tracking-td-label);
  text-transform: uppercase;
  color: var(--color-td-gray-600);
  background: none;
  border: none;
  cursor: pointer;
  transition: color 0.15s;
}

.td-content-nav-label:hover {
  color: var(--color-td-fg);
}

.td-content-nav-caret {
  flex-shrink: 0;
  transition: transform 0.15s;
}

.td-content-nav-caret.is-collapsed {
  transform: rotate(-90deg);
}

.td-content-nav-link {
  display: block;
  padding: 5px 20px 5px 32px;
  font-size: var(--font-size-td-nav);
  color: var(--color-td-gray-700);
  text-decoration: none;
  border-left: 3px solid transparent;
  transition: background-color 0.1s;
}

.td-content-nav-link:hover {
  background-color: var(--color-td-gray-200);
}

.td-content-nav-link[aria-current="page"] {
  border-left-color: var(--color-td-primary);
  font-weight: var(--font-weight-td-semibold);
  color: var(--color-td-fg);
}
</style>
