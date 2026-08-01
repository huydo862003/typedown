import {
  computed,
} from 'vue';
import {
  createInjectionState,
} from '@vueuse/core';
import type {
  Route,
} from '../router';
import {
  useTheme,
} from './useTheme';
import type {
  TypedownData,
} from '@/shared';

export const [
  provideTdContent,
  useTdContent,
] = createInjectionState(_useTdContent);

// Create reactive data bindings from the current route
function _useTdContent (route: Route): TypedownData {
  const theme = useTheme();

  return {
    page: computed(() => route.data),
    frontmatter: computed(() => route.data.frontmatter),
    title: computed(() => route.data.title),
    isDark: computed(() => theme.isDark.value),
  };
}
