import { describe, expect, it } from 'vitest';

import { addRecent, MAX_RECENT, parseRecent, removeRecent, serializeRecent } from './project';

describe('recent projects', () => {
  it('puts the newest first and dedupes by path', () => {
    let l = addRecent([], '/a', 1);
    l = addRecent(l, '/b', 2);
    l = addRecent(l, '/a', 3);
    expect(l).toEqual([{ path: '/a', openedAt: 3 }, { path: '/b', openedAt: 2 }]);
  });

  it('caps the list at 8 and drops the oldest', () => {
    let l: ReturnType<typeof addRecent> = [];
    for (let i = 0; i < 12; i++) l = addRecent(l, `/p${i}`, i);
    expect(l).toHaveLength(MAX_RECENT);
    expect(l[0].path).toBe('/p11');
    expect(l.some((r) => r.path === '/p3')).toBe(false);
  });

  it('refuses an empty path', () => {
    expect(addRecent([{ path: '/a', openedAt: 1 }], '', 2)).toEqual([{ path: '/a', openedAt: 1 }]);
  });

  it('removes one entry and leaves the rest', () => {
    const l = [{ path: '/a', openedAt: 2 }, { path: '/b', openedAt: 1 }];
    expect(removeRecent(l, '/a')).toEqual([{ path: '/b', openedAt: 1 }]);
    expect(removeRecent(l, '/zzz')).toEqual(l);
  });

  it('round-trips and rejects malformed storage', () => {
    const l = [{ path: '/a', openedAt: 2 }];
    expect(parseRecent(serializeRecent(l))).toEqual(l);
    expect(parseRecent(null)).toEqual([]);
    expect(parseRecent('not json')).toEqual([]);
    expect(parseRecent('{"path":"/a"}')).toEqual([]);
    expect(parseRecent('[{"path":1,"openedAt":1},{"path":"/ok","openedAt":5},null,{"path":"/x"}]')).toEqual([{ path: '/ok', openedAt: 5 }]);
  });

  it('keeps the newest of duplicates and caps what it reads', () => {
    expect(parseRecent('[{"path":"/a","openedAt":1},{"path":"/a","openedAt":9}]')).toEqual([{ path: '/a', openedAt: 9 }]);
    const many = Array.from({ length: 20 }, (_, i) => ({ path: `/p${i}`, openedAt: i }));
    expect(parseRecent(JSON.stringify(many))).toHaveLength(MAX_RECENT);
  });
});
