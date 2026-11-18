import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {checkRetrieval} from './retrieval-check.mjs';
const id = 'a'.repeat(64), hash = 'b'.repeat(64), root = `/releases/${id}/`;
// `prepare` mutates the routes before the manifest is hashed, so the release
// stays integrity-valid; `change` mutates them afterwards.
async function fixture(t, change = () => {}, prepare = () => {}) {
  const hits = [];
  const routes = {
    '/llms.txt': `[Selected corpus](${root}index.md)`,
    [`${root}index.md`]: '[LRITF](mx/lritf/index.md)',
    [`${root}mx/lritf/index.md`]: `[Artículo 135](provision-${'c'.repeat(64)}.md)\n[Artículo 35](provision-${hash}.md)\n[Artículo 39 Bis](provision-${'d'.repeat(64)}.md)\n[Artículo 39](provision-${'e'.repeat(64)}.md)`,
    [`${root}mx/lritf/provision-${hash}.md`]: 'private fixture legal body',
    [`${root}mx/lritf/provision-${'e'.repeat(64)}.md`]: 'another body',
    [`${root}manifest.json`]: '{}',
  };
  prepare(routes);
  routes[`${root}mx/lritf/provision-${hash}.txt`] = 'private fixture legal body';
  routes[`${root}mx/lritf/provision-${'e'.repeat(64)}.txt`] = 'another body';
  const digest = b => createHash('sha256').update(b).digest('hex');
  const payload = {schema_version: 'public-release-v1', base_path: '/', files: Object.entries(routes).filter(([p]) => p.startsWith(root) && !p.endsWith('manifest.json')).map(([path, body]) => ({path: path.slice(root.length), bytes: Buffer.byteLength(body), sha256: digest(body)}))};
  const releaseId = digest(JSON.stringify(payload, null, 2) + '\n');
  routes[`${root}manifest.json`] = JSON.stringify({...payload, release_id: releaseId});
  change(routes);
  for (const [path, body] of Object.entries(routes)) {
    delete routes[path];
    routes[path.replace(id, releaseId)] = typeof body === 'string' ? body.replaceAll(id, releaseId) : body;
  }
  const server = createServer((req, res) => {
    hits.push({url: req.url, method: req.method, ua: req.headers['user-agent']});
    const route = routes[req.url];
    if (typeof route === 'function') return route(req, res);
    res.writeHead(route === undefined ? 404 : 200, {'content-type': 'text/markdown', 'cache-control': 'public, max-age=30', 'set-cookie': 'secret'});
    res.end(route);
  });
  await new Promise(r => server.listen(0, '127.0.0.1', r));
  t.after(() => { server.closeAllConnections(); server.close(); });
  return {releaseId, baseUrl: `http://127.0.0.1:${server.address().port}/`, hits};
}
test('discovers actual exact articles and records bounded GET/HEAD evidence without bodies or cookies', async t => {
  const f = await fixture(t);
  const report = await checkRetrieval({...f, userAgent: 'test-client'});
  assert.equal(report.ok, true);
  assert.equal(report.releaseId, f.releaseId);
  assert.equal(report.discovery.requests.length, 10);
  assert.equal(report.verification.requests.length, 6);
  assert.deepEqual(report.provisions.map(p => p.article), ['35', '39']);
  assert.equal(report.discovery.requests[0].cache['cache-control'], 'public, max-age=30');
  assert.match(report.provisions[0].sha256, /^[a-f0-9]{64}$/);
  assert.ok(f.hits.every(h => h.ua === 'test-client'));
  assert.ok(!JSON.stringify(report).includes('private fixture legal body'));
  assert.ok(!JSON.stringify(report).includes('secret'));
});
test('HTTP errors are separate from transport failures', async t => {
  const f = await fixture(t, r => { delete r['/llms.txt']; });
  const report = await checkRetrieval(f);
  assert.equal(report.errors[0].kind, 'http');
  assert.equal(report.discovery.requests[0].status, 404);
  assert.ok(report.discovery.requests[0].sha256);
});
test('timeout is bounded and reported distinctly', async t => {
  const f = await fixture(t, r => { r['/llms.txt'] = () => {}; });
  const report = await checkRetrieval({...f, timeoutMs: 30});
  assert.equal(report.errors[0].kind, 'timeout');
});
test('rejects missing or ambiguous exact article labels', async t => {
  const f = await fixture(t, r => { r[`${root}mx/lritf/index.md`] = `[Artículo 135](provision-${hash}.md)`; });
  const report = await checkRetrieval(f);
  assert.match(report.errors[0].message, /exact article/);
});
test('blocks cross-origin links before fetching', async t => {
  const f = await fixture(t, r => { r['/llms.txt'] = `[Selected corpus](http://example.invalid${root}index.md)`; });
  const report = await checkRetrieval(f);
  assert.equal(report.ok, false);
  assert.equal(f.hits.length, 2);
});
test('blocks redirects leaving pinned release and captures redirects', async t => {
  const f = await fixture(t, r => { r[`${root}index.md`] = (_q, s) => { s.writeHead(302, {location: '/other/index.md'}); s.end(); }; });
  const report = await checkRetrieval(f);
  assert.equal(report.ok, false);
  assert.equal(report.discovery.requests.at(-1).redirects[0].status, 302);
  assert.ok(!f.hits.some(h => h.url === '/other/index.md'));
});
for (const [option, value] of [['maxRequests', 2], ['maxBytes', 10], ['maxTotalBytes', 10]]) {
  test(`enforces ${option}`, async t => {
    const f = await fixture(t);
    const report = await checkRetrieval({...f, [option]: value});
    assert.equal(report.errors[0].kind, 'limit');
  });
}
test('rejects malformed manifest for optional full verification', async t => {
  const f = await fixture(t, r => { r[`${root}manifest.json`] = '{}'; });
  const report = await checkRetrieval({...f, verifyManifest: true});
  assert.equal(report.ok, false);
  assert.match(report.errors[0].message, /manifest/);
  assert.equal(report.discovery.requests.length, 10);
});
test('transport errors preserve status null', async () => {
  const server = createServer();
  await new Promise(r => server.listen(0, '127.0.0.1', r));
  const port = server.address().port;
  await new Promise(r => server.close(r));
  const report = await checkRetrieval({baseUrl: `http://127.0.0.1:${port}/`});
  assert.equal(report.errors[0].kind, 'transport');
  assert.equal(report.discovery.requests[0].status, null);
});

test('HEAD405 is visible while GET discovery and integrity verification continue', async t => {
  const f = await fixture(t, routes => {
    const body = routes[`${root}index.md`];
    routes[`${root}index.md`] = (req, res) => { res.writeHead(req.method === 'HEAD' ? 405 : 200); res.end(req.method === 'HEAD' ? '' : body); };
  });
  const report = await checkRetrieval(f);
  assert.equal(report.ok, false);
  assert.equal(report.discoveryCompleted, true);
  assert.equal(report.verification.verifiedFiles, 6);
  assert.ok(report.errors.some(e => e.kind === 'http'));
});
test('manifest verification detects tampered legal body', async t => {
  const f = await fixture(t, routes => { routes[`${root}mx/lritf/provision-${hash}.md`] += 'tamper'; });
  const report = await checkRetrieval(f);
  assert.equal(report.ok, false);
  assert.match(report.errors[0].message, /integrity mismatch/);
});
test('optional full manifest verification succeeds independently of discovery budget', async t => {
  const f = await fixture(t);
  const report = await checkRetrieval({...f, maxRequests: 10, verifyManifest: true});
  assert.equal(report.ok, true);
  assert.equal(report.verification.fullManifestVerified, true);
  assert.equal(report.verification.verifiedFiles, 6);
});

test('rejects within-release wrong-article redirect against requested manifest row', async t => {
  const f = await fixture(t, routes => {
    routes[`${root}mx/lritf/provision-${hash}.md`] = (req, res) => {
      res.writeHead(302, {location: `provision-${'e'.repeat(64)}.md`}); res.end();
    };
  });
  const report = await checkRetrieval(f);
  assert.equal(report.discoveryCompleted, true);
  assert.equal(report.ok, false);
  assert.equal(report.errors[0].kind, 'verification');
  assert.match(report.errors[0].message, /integrity mismatch/);
});
// Codex audit 2026-11-17, R2: unrelated links on the release index used to
// abort discovery because the strict pin threw inside the link filter.
for (const [name, extra] of [['site home', '[Inicio](/)'], ['official DOF', '[DOF](https://www.dof.gob.mx/)'], ['anchor', '[Arriba](#top)'], ['query', '[Todo](?view=all)']]) {
  test(`an unrelated ${name} link on the index does not abort discovery`, async t => {
    const f = await fixture(t, r => { r[`${root}index.md`] = `${extra}\n[LRITF](mx/lritf/index.md)`; });
    const report = await checkRetrieval(f);
    assert.ok(!report.errors.some(e => /escapes allowed origin/.test(e.message)), JSON.stringify(report.errors));
    assert.deepEqual(report.provisions.map(p => p.article), ['35', '39']);
  });
}
// Codex audit re-review 2026-11-17, F2: extra navigation to the same index path
// (a fragment or an alternate view) is not a second eligible index link.
for (const [name, extra] of [['fragment', '[Extra navigation](mx/lritf/index.md#toc)'], ['query', '[Extra navigation](mx/lritf/index.md?view=all)']]) {
  test(`a same-index ${name} link beside the canonical link does not abort discovery`, async t => {
    const f = await fixture(t, r => { r[`${root}index.md`] = `${extra}\n[LRITF](mx/lritf/index.md)`; });
    const report = await checkRetrieval(f);
    assert.deepEqual(report.provisions?.map(p => p.article), ['35', '39'], JSON.stringify(report.errors));
    assert.equal(report.discoveryCompleted, true, JSON.stringify(report.errors));
    // The variant is not eligible, so it is never requested.
    assert.ok(!f.hits.some(h => /index\.md[?#]/.test(h.url)), JSON.stringify(f.hits));
  });
}
// Codex audit re-review 2026-11-17, F2 (second pass): a bare `?` or `#` parses
// with an empty `search`/`hash`, so component checks cannot tell it from an
// absent component. The selection compares the exact serialized URL instead.
// These releases are hashed with the extra link in place, so the full run
// (including manifest verification) must succeed.
for (const [name, extra] of [['bare query', '[Navigation](mx/lritf/index.md?)'], ['bare fragment', '[Navigation](mx/lritf/index.md#)']]) {
  test(`a same-index ${name} link beside the canonical link does not abort discovery`, async t => {
    const f = await fixture(t, () => {}, r => { r[`${root}index.md`] = `${extra}\n[LRITF](mx/lritf/index.md)`; });
    const report = await checkRetrieval(f);
    assert.equal(report.ok, true, JSON.stringify(report.errors));
    assert.equal(report.discoveryCompleted, true);
    assert.deepEqual(report.provisions.map(p => p.article), ['35', '39']);
    assert.ok(!f.hits.some(h => /index\.md[?#]/.test(h.url)), JSON.stringify(f.hits));
  });
}
// The strict pin applies to the selected fetch: a bare delimiter on a followed
// link is rejected before any request, as a nonempty one already was.
for (const [name, suffix] of [['bare query', '?'], ['bare fragment', '#'], ['query', '?x=1'], ['fragment', '#top']]) {
  test(`a followed article link with a ${name} is rejected before it is requested`, async t => {
    const f = await fixture(t, () => {}, r => {
      r[`${root}mx/lritf/index.md`] = r[`${root}mx/lritf/index.md`].replace(`provision-${hash}.md)`, `provision-${hash}.md${suffix})`);
    });
    const report = await checkRetrieval(f);
    assert.equal(report.ok, false);
    assert.match(report.errors[0].message, /escapes allowed origin/);
    assert.ok(!f.hits.some(h => h.url.includes(`provision-${hash}`)), JSON.stringify(f.hits));
  });
}
test('a credentialed same-index link is not eligible and is never followed', async t => {
  // The port is known only once the server listens, so the index is built per request.
  const f = await fixture(t, r => {
    r[`${root}index.md`] = (req, res) => {
      res.writeHead(200, {'content-type': 'text/markdown'});
      res.end(`[Credentialed](http://user:pw@${req.headers.host}${req.url.replace(/index\.md$/, 'mx/lritf/index.md')})\n[LRITF](mx/lritf/index.md)`);
    };
  });
  const report = await checkRetrieval(f);
  assert.deepEqual(report.provisions?.map(p => p.article), ['35', '39'], JSON.stringify(report.errors));
  assert.equal(report.discoveryCompleted, true, JSON.stringify(report.errors));
});
test('an LRITF-looking link on another origin is still never followed', async t => {
  const f = await fixture(t, r => { r[`${root}index.md`] = `[LRITF](http://example.invalid${root}mx/lritf/index.md)`; });
  const report = await checkRetrieval(f);
  assert.equal(report.ok, false);
  assert.match(report.errors[0].message, /exactly one LRITF index link/);
  assert.ok(!f.hits.some(h => h.url.includes('mx/lritf/index.md')));
});
