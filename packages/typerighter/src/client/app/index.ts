import {
  createApp,
  defineComponent,
  h,
  inject,
  type App,
  type Component,
  type InjectionKey,
} from 'vue';
import {
  Content,
} from './components/Content';
import {
  provideTdContent,
} from './composables/useTdContent';
import {
  createRouter, routerSymbol, type Router,
} from './router';
import {
  isInBrowser,
  type PageModule,
  type SidebarGroups,
} from '@/shared';

export {
  useTdContent,
} from './composables/useTdContent';
export {
  useRouter, useRoute,
} from './router';
export {
  Content,
} from './components/Content';

export interface TypedownSiteConfig {
  /** Site title */
  title: string;
  /** Site description */
  description: string;
}

export interface TypedownSiteData {
  /** Content files grouped by schema */
  sidebarGroups: SidebarGroups;
}

const siteConfigSymbol: InjectionKey<TypedownSiteConfig> = Symbol('typedown-site-config');
const siteDataSymbol: InjectionKey<TypedownSiteData> = Symbol('typedown-site-data');

export async function createTypedownApp (
  loadPageModule: (path: string) => Promise<PageModule | undefined>,
  Layout: Component,
  config: Partial<TypedownSiteConfig> = {},
  data: Partial<TypedownSiteData> = {},
): Promise<{
  app: App;
  router: Router;
}> {
  const siteConfig: TypedownSiteConfig = {
    title: config.title ?? '',
    description: config.description ?? '',
  };

  const siteData: TypedownSiteData = {
    sidebarGroups: data.sidebarGroups ?? {},
  };

  const router = createRouter(loadPageModule);

  const TypedownApp = defineComponent({
    name: 'TypedownApp',
    setup () {
      provideTdContent(router.route);

      return () => h(Layout);
    },
  });

  const app = createApp(TypedownApp);

  app.provide(routerSymbol, router);
  app.provide(siteConfigSymbol, siteConfig);
  app.provide(siteDataSymbol, siteData);
  app.component('TypedownContent', Content);

  if (isInBrowser) {
    await router.go(location.href, {
      replace: true,
      initialLoad: true,
    });
  }

  return {
    app,
    router,
  };
}

export function useSiteConfig (): TypedownSiteConfig {
  return inject(siteConfigSymbol, {
    title: '',
    description: '',
  });
}

export function useSiteData (): TypedownSiteData {
  return inject(siteDataSymbol, {
    sidebarGroups: {},
  });
}
