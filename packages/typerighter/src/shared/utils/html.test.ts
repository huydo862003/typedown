import {
  describe, it, expect,
} from 'vitest';
import {
  escapeHtml,
} from './html';

describe('escapeHtml', () => {
  it('escapes ampersands', () => {
    expect(escapeHtml('a & b')).toBe('a &amp; b');
  });

  it('escapes double quotes', () => {
    expect(escapeHtml('"hello"')).toBe('&quot;hello&quot;');
  });

  it('escapes angle brackets', () => {
    expect(escapeHtml('<script>')).toBe('&lt;script&gt;');
  });

  it('escapes all special characters together', () => {
    expect(escapeHtml('<a href="&">')).toBe('&lt;a href=&quot;&amp;&quot;&gt;');
  });

  it('returns plain strings unchanged', () => {
    expect(escapeHtml('hello world')).toBe('hello world');
  });
});
