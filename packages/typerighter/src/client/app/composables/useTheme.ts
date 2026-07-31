import {
  ref, computed, readonly,
} from 'vue';

export type Theme = 'light' | 'dark' | 'system';

const STORAGE_KEY = 'td-theme';

// Apply the dark class to the document root
function applyTheme (theme: Theme): void {
  if (typeof document === 'undefined') return;

  const effective = theme === 'system' ? getSystemTheme() : theme;

  document.documentElement.classList.toggle('dark', effective === 'dark');
}

// Resolve the system preference
function getSystemTheme (): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'light';

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

// Shared reactive state so all consumers see the same value
const theme = ref<Theme>('system');
const systemTheme = ref<'light' | 'dark'>(getSystemTheme());
let initialized = false;

// Shared theme composable with localStorage persistence
export function useTheme () {
  initialize();

  const effectiveTheme = computed<'light' | 'dark'>(() =>
    theme.value === 'system' ? systemTheme.value : theme.value);

  const isDark = computed(() => effectiveTheme.value === 'dark');

  function setTheme (newTheme: Theme): void {
    theme.value = newTheme;
    applyTheme(newTheme);

    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, newTheme);
    }
  }

  function toggle (): void {
    setTheme(effectiveTheme.value === 'light' ? 'dark' : 'light');
  }

  return {
    theme: readonly(theme),
    effectiveTheme,
    isDark,
    setTheme,
    toggle,
  };
}

// One-time setup for localStorage and system preference listener
function initialize (): void {
  if (initialized) return;
  initialized = true;

  if (typeof localStorage !== 'undefined') {
    const stored = localStorage.getItem(STORAGE_KEY) as string | null;

    if (stored === 'light' || stored === 'dark' || stored === 'system') {
      theme.value = stored;
    }
  }

  applyTheme(theme.value);

  if (typeof window !== 'undefined') {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    mediaQuery.addEventListener('change', () => {
      systemTheme.value = getSystemTheme();

      if (theme.value === 'system') applyTheme('system');
    });
  }
}
