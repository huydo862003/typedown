import {
  describe, it, expect,
} from 'vitest';
import {
  generateAppEntry, generateIndexSfc, generateSsrEntry,
} from './codegen';

describe('generateAppEntry', () => {
  it('generates import.meta.glob for .td files in content directory', () => {
    const result = generateAppEntry({
      contentDir: 'content',
      siteTitle: '',
      siteDescription: '',
      hasIndex: true,
      sidebarGroups: {},
    });

    expect(result).toContain('import.meta.glob(\'/content/**/*.td\')');
  });

  it('imports createTypedownApp from typerighter/client', () => {
    const result = generateAppEntry({
      contentDir: 'content',
      siteTitle: '',
      siteDescription: '',
      hasIndex: true,
      sidebarGroups: {},
    });

    expect(result).toContain('import { createTypedownApp } from \'typerighter/client\'');
  });

  it('imports theme from typerighter/client/theme-default', () => {
    const result = generateAppEntry({
      contentDir: 'content',
      siteTitle: '',
      siteDescription: '',
      hasIndex: true,
      sidebarGroups: {},
    });

    expect(result).toContain('import theme from \'typerighter/client/theme-default\'');
  });

  it('mounts the app on #app', () => {
    const result = generateAppEntry({
      contentDir: 'content',
      siteTitle: '',
      siteDescription: '',
      hasIndex: true,
      sidebarGroups: {},
    });

    expect(result).toContain('app.mount(\'#app\')');
  });

  it('does not inject virtual index when index.td exists', () => {
    const result = generateAppEntry({
      contentDir: 'content',
      siteTitle: '',
      siteDescription: '',
      hasIndex: true,
      sidebarGroups: {},
    });

    expect(result).not.toContain('__typedown_index__');
  });

  it('injects virtual index fallback when no index.td exists', () => {
    const result = generateAppEntry({
      contentDir: 'content',
      siteTitle: '',
      siteDescription: '',
      hasIndex: false,
      sidebarGroups: {},
    });

    expect(result).toContain('__typedown_index__');
    expect(result).toContain('pagePath === \'/\'');
  });

  it('uses the provided content directory in glob and keys', () => {
    const result = generateAppEntry({
      contentDir: 'docs',
      siteTitle: '',
      siteDescription: '',
      hasIndex: true,
      sidebarGroups: {},
    });

    expect(result).toContain('import.meta.glob(\'/docs/**/*.td\')');
    expect(result).toContain('\'/docs/\'');
  });
});

describe('generateIndexSfc', () => {
  it('returns a valid Vue SFC with script and template', () => {
    const result = generateIndexSfc({});

    expect(result).toContain('<script>');
    expect(result).toContain('</script>');
    expect(result).toContain('<template>');
    expect(result).toContain('</template>');
  });

  it('imports TdOverview from theme-default', () => {
    const result = generateIndexSfc({});

    expect(result).toContain('import { TdOverview } from \'typerighter/client/theme-default\'');
  });

  it('exports __pageData with Index title', () => {
    const result = generateIndexSfc({});

    expect(result).toContain('__pageData');
    expect(result).toContain('Index');
  });

  it('passes groups data as prop', () => {
    const groups = {
      Person: [
        {
          path: 'people/alice.td',
          schema: 'Person',
          header: {
            name: 'Alice',
          },
        },
      ],
      Project: [
        {
          path: 'projects/web.td',
          schema: 'Project',
          header: {
            name: 'Web',
          },
        },
      ],
    };
    const result = generateIndexSfc(groups);

    expect(result).toContain('Alice');
    expect(result).toContain('Web');
    expect(result).toContain(':groups=');
  });

  it('escapes single quotes in data', () => {
    const groups = {
      Test: [
        {
          path: 'test.td',
          schema: 'Test',
          header: {
            name: 'O\'Brien',
          },
        },
      ],
    };
    const result = generateIndexSfc(groups);

    expect(result).toContain('O\\\'Brien');
    expect(result).not.toContain('O\'Brien');
  });

  it('handles empty groups', () => {
    const result = generateIndexSfc({});

    expect(result).toContain(':groups=\'{}\'');
  });
});

describe('generateSsrEntry', () => {
  it('uses eager glob for SSR', () => {
    const result = generateSsrEntry({
      contentDir: 'vault/content',
      layoutImport: 'typerighter/client/theme-default',
      siteConfig: '{}',
      siteData: '{}',
      hasIndex: false,
    });

    expect(result).toContain('import.meta.glob(\'/vault/content/**/*.td\', { eager: true })');
  });

  it('imports renderToString from vue/server-renderer', () => {
    const result = generateSsrEntry({
      contentDir: 'content',
      layoutImport: 'typerighter/client/theme-default',
      siteConfig: '{}',
      siteData: '{}',
      hasIndex: false,
    });

    expect(result).toContain('import { renderToString } from \'vue/server-renderer\'');
  });

  it('imports theme from the provided layout import', () => {
    const result = generateSsrEntry({
      contentDir: 'content',
      layoutImport: 'my-custom-theme',
      siteConfig: '{}',
      siteData: '{}',
      hasIndex: false,
    });

    expect(result).toContain('import theme from \'my-custom-theme\'');
  });

  it('exports a render function', () => {
    const result = generateSsrEntry({
      contentDir: 'content',
      layoutImport: 'typerighter/client/theme-default',
      siteConfig: '{}',
      siteData: '{}',
      hasIndex: false,
    });

    expect(result).toContain('export async function render(url)');
  });

  it('passes siteConfig and siteData to createTypedownApp', () => {
    const result = generateSsrEntry({
      contentDir: 'content',
      layoutImport: 'typerighter/client/theme-default',
      siteConfig: '{"title":"Test"}',
      siteData: '{"sidebarGroups":{}}',
      hasIndex: false,
    });

    expect(result).toContain('{"title":"Test"}');
    expect(result).toContain('{"sidebarGroups":{}}');
  });
});
