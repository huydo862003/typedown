import {
  describe, it, expect,
} from 'vitest';
import {
  generateAppEntry, generateIndexSfc,
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

  it('imports ContentIndex from theme-default', () => {
    const result = generateIndexSfc({});

    expect(result).toContain('import { ContentIndex } from \'typerighter/client/theme-default\'');
  });

  it('exports __pageData with Content Index title', () => {
    const result = generateIndexSfc({});

    expect(result).toContain('__pageData');
    expect(result).toContain('Content Index');
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
