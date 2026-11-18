import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const links = text => [...text.matchAll(/\[([^\]\n]+)\]\(([^\s)]+)\)/g)].map(([, label, href]) => ({label, href}));
const positive = (value, name) => {
  if (!Number.isSafeInteger(value) || value < 1) throw new Error(`${name} must be a positive integer`);
  return value;
};

/** Network evidence only; bodies remain internal and are never included in the report. */
export async function checkRetrieval(options = {}) {
  const base = new URL(options.baseUrl);
  if (!['http:', 'https:'].includes(base.protocol) || base.username || base.password || base.search || base.hash) throw new Error('Invalid base URL');
  if (!base.pathname.endsWith('/')) base.pathname += '/';
  const articles = options.articles ?? ['35', '39'];
  if (!Array.isArray(articles) || !articles.length || articles.some(n => !/^\d+(?: Bis| Ter| Quáter)?$/.test(n)) || new Set(articles).size !== articles.length) throw new Error('Invalid article locators');
  const limits = {
    timeoutMs: positive(options.timeoutMs ?? 10000, 'timeoutMs'),
    maxBytes: positive(options.maxBytes ?? 2_000_000, 'maxBytes'),
    maxRequests: positive(options.maxRequests ?? 24, 'maxRequests'),
    maxTotalBytes: positive(options.maxTotalBytes ?? 8_000_000, 'maxTotalBytes'),
  };
  const report = {schemaVersion: 'retrieval-evidence-v1', scope: 'HTTP transport and published artifact integrity; not legal accuracy or canonical article identity', startedAt: new Date().toISOString(), baseUrl: base.href,
    articles, runtime: {node: process.version, platform: process.platform}, userAgent: options.userAgent ?? null, limits, discovery: {requests: [], requestCount: 0, bytes: 0}, verification: {enabled: true, fullManifestRequested: !!options.verifyManifest, requests: [], requestCount: 0, bytes: 0}, errors: []};
  let root;
  const safe = (value, parent, pinned = true) => {
    const url = new URL(value, parent);
    if (url.origin !== base.origin || url.username || url.password || url.search || url.hash || !url.pathname.startsWith(pinned ? root : base.pathname)) throw Object.assign(new Error('URL escapes allowed origin or pinned release'), {kind: 'discovery'});
    return url;
  };
  async function request(value, method, section, pinned = true) {
    const bucket = report[section];
    const maxRequests = section === 'verification' ? positive(options.verificationMaxRequests ?? 2000, 'verificationMaxRequests') : limits.maxRequests;
    const maxTotal = section === 'verification' ? positive(options.verificationMaxBytes ?? 128_000_000, 'verificationMaxBytes') : limits.maxTotalBytes;
    const entry = {method, url: String(value), redirects: [], status: null, finalUrl: null, mime: null, cache: {}, bytes: 0, sha256: null, elapsedMs: 0, error: null};
    let bytes = Buffer.alloc(0);
    const start = performance.now();
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), limits.timeoutMs);
    try {
      let url = safe(value, base, pinned);
      for (let hops = 0; ; hops++) {
        if (bucket.requestCount >= maxRequests) throw Object.assign(new Error('Request count limit exceeded'), {kind: 'limit'});
        // Each HTTP exchange, including redirects, consumes the request budget.
        if (!bucket.requests.includes(entry)) bucket.requests.push(entry);
        bucket.requestCount++;
        const response = await fetch(url, {method, redirect: 'manual', signal: controller.signal, headers: options.userAgent ? {'user-agent': options.userAgent} : {}});
        entry.status = response.status;
        entry.finalUrl = url.href;
        entry.mime = response.headers.get('content-type');
        entry.cache = Object.fromEntries(['cache-control', 'etag', 'last-modified', 'age', 'expires', 'vary', 'cf-cache-status', 'content-length', 'server', 'cf-ray'].map(k => [k, response.headers.get(k)]));
        if ([301, 302, 303, 307, 308].includes(response.status)) {
          const location = response.headers.get('location');
          entry.redirects.push({url: url.href, status: response.status, location, mime: entry.mime, cache: entry.cache});
          await response.body?.cancel();
          if (!location || hops >= 5) throw Object.assign(new Error('Invalid or excessive redirects'), {kind: 'discovery'});
          url = safe(location, url, pinned);
          continue;
        }
        const chunks = [];
        if (response.body) for await (const chunk of response.body) {
          entry.bytes += chunk.length;
          bucket.bytes += chunk.length;
          if (entry.bytes > limits.maxBytes || bucket.bytes > maxTotal) {
            controller.abort();
            throw Object.assign(new Error('Response byte limit exceeded'), {kind: 'limit'});
          }
          chunks.push(chunk);
        }
        bytes = Buffer.concat(chunks);
        entry.sha256 = sha(bytes);
        if (!response.ok) throw Object.assign(new Error(`HTTP ${response.status}`), {kind: 'http'});
        break;
      }
    } catch (error) {
      entry.error = {kind: error.kind ?? (controller.signal.aborted ? 'timeout' : 'transport'), message: error.message};
    } finally {
      clearTimeout(timer);
      entry.elapsedMs = Math.round((performance.now() - start) * 100) / 100;
    }
    if (entry.error) throw Object.assign(new Error(entry.error.message), {evidence: entry.error});
    return bytes;
  }
  async function pair(url, section = 'discovery', pinned = true) {
    let bytes, failure;
    try { bytes = await request(url, 'GET', section, pinned); } catch (error) { failure = error; }
    try { await request(url, 'HEAD', section, pinned); } catch (error) { report.errors.push(error.evidence ?? {kind: 'transport', message: error.message}); }
    if (failure) throw failure;
    return bytes;
  }
  try {
    const discovery = (await pair(new URL('llms.txt', base), 'discovery', false)).toString('utf8');
    const candidates = links(discovery).filter(l => /\/releases\/[a-f0-9]{64}\/index\.md$/.test(new URL(l.href, base).pathname));
    if (candidates.length !== 1) throw new Error('Expected exactly one release index link in llms.txt');
    const indexUrl = safe(candidates[0].href, base, false);
    report.releaseId = indexUrl.pathname.match(/\/releases\/([a-f0-9]{64})\/index\.md$/)[1];
    root = indexUrl.pathname.slice(0, -'index.md'.length);
    const index = (await pair(indexUrl)).toString('utf8');
    // Classify without throwing: an unrelated link on the index (site home,
    // an official DOF address, an anchor) is skipped, not fatal. The strict
    // pin still applies to the one link selected (Codex audit 2026-11-17, R2).
    const resolves = (href, parent) => { try { return new URL(href, parent); } catch { return null; } };
    // Eligible means the exact index URL: same origin and pathname, with no
    // query, fragment or credentials. Extra navigation to the same index (a
    // table-of-contents anchor, an alternate view) is not a second candidate
    // (Codex audit re-review 2026-11-17, F2).
    const instruments = links(index).filter(l => {
      const target = resolves(l.href, indexUrl);
      return target !== null && target.origin === base.origin && target.pathname === `${root}mx/lritf/index.md`
        && target.search === '' && target.hash === '' && target.username === '' && target.password === '';
    });
    if (instruments.length !== 1) throw new Error('Expected exactly one LRITF index link');
    const instrumentUrl = safe(instruments[0].href, indexUrl);
    const instrument = (await pair(instrumentUrl)).toString('utf8');
    report.provisions = [];
    for (const article of articles) {
      const matches = links(instrument).filter(l => l.label.normalize('NFC') === `Artículo ${article}`);
      if (matches.length !== 1) throw new Error(`Expected exactly one exact article link: ${article}`);
      const url = safe(matches[0].href, instrumentUrl);
      if (!url.pathname.startsWith(`${root}mx/lritf/`) || !/^provision-[a-f0-9]{64}\.md$/.test(url.pathname.slice(`${root}mx/lritf/`.length))) throw new Error('Invalid provision artifact path');
      const bytes = await pair(url);
      report.provisions.push({article, url: url.href, bytes: bytes.length, sha256: sha(bytes)});
    }
    report.discoveryCompleted = true;
    const manifestUrl = new URL('manifest.json', indexUrl);
    report.manifestUrl = manifestUrl.href;
    const manifestBytes = await pair(manifestUrl, 'verification');
    const manifest = JSON.parse(manifestBytes.toString('utf8'));
    if (manifest.release_id !== report.releaseId || manifest.schema_version !== 'public-release-v1' || `${manifest.base_path}releases/${report.releaseId}/` !== root || !Array.isArray(manifest.files)) throw new Error('Invalid pinned manifest');
    const payload = {...manifest}; delete payload.release_id;
    if (sha(Buffer.from(JSON.stringify(payload, null, 2) + '\n')) !== report.releaseId) throw new Error('Manifest content ID mismatch');
    const files = new Map();
    for (const file of manifest.files) {
      if (!file || typeof file.path !== 'string' || !/^(?:index\.md|coverage\.json|sources\/[a-f0-9]{64}\.pdf|mx\/[a-z0-9-]+\/(?:index\.md|instrument\.json|provision-[a-f0-9]{64}\.(?:md|txt)))$/.test(file.path) || files.has(file.path) || !Number.isSafeInteger(file.bytes) || file.bytes < 0 || !/^[a-f0-9]{64}$/.test(file.sha256)) throw new Error('Invalid or duplicate manifest file');
      files.set(file.path, file);
    }
    const verified = new Set();
    const verify = (url, bytes, digest) => {
      const path = new URL(url).pathname.slice(root.length), file = files.get(path);
      if (!file || bytes !== file.bytes || digest !== file.sha256) throw new Error(`Manifest integrity mismatch: ${path}`);
      verified.add(path);
    };
    for (const entry of report.discovery.requests.filter(e => e.method === 'GET' && !e.error && e.url !== new URL('llms.txt', base).href)) verify(entry.url, entry.bytes, entry.sha256);
    // public-release-v1 explicitly publishes an exact-text .txt companion for each .md provision.
    for (const provision of report.provisions) {
      const textUrl = provision.url.replace(/\.md$/, '.txt');
      const bytes = await pair(textUrl, 'verification');
      verify(textUrl, bytes.length, sha(bytes));
      provision.exactText = {url: textUrl, bytes: bytes.length, sha256: sha(bytes), manifestVerified: true};
    }
    if (options.verifyManifest) for (const [path] of files) {
      if (verified.has(path)) continue;
      const url = safe(path, indexUrl);
      const bytes = await request(url, 'GET', 'verification');
      verify(url, bytes.length, sha(bytes));
    }
    report.verification.verifiedFiles = verified.size;
    report.verification.fullManifestVerified = !!options.verifyManifest;

  } catch (error) {
    report.errors.push(error.evidence ?? {kind: report.discoveryCompleted ? 'verification' : 'discovery', message: error.message});
  }
  report.discoveryCompleted ??= false;
  report.stats = Object.fromEntries(['discovery', 'verification'].map(section => [section, {get: report[section].requests.filter(r => r.method === 'GET').length, head: report[section].requests.filter(r => r.method === 'HEAD').length, httpExchanges: report[section].requestCount, bytes: report[section].bytes}]));
  report.ok = report.errors.length === 0;
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try {
    const args = process.argv.slice(2), options = {};
    const flags = {'--base-url': 'baseUrl', '--user-agent': 'userAgent', '--articles': 'articles', '--timeout-ms': 'timeoutMs', '--max-bytes': 'maxBytes', '--max-requests': 'maxRequests', '--max-total-bytes': 'maxTotalBytes', '--verification-max-requests': 'verificationMaxRequests', '--verification-max-bytes': 'verificationMaxBytes'};
    for (let i = 0; i < args.length; i++) {
      if (args[i] === '--verify-manifest') { options.verifyManifest = true; continue; }
      const key = flags[args[i]];
      if (!key || !args[i + 1]) throw new Error('Usage: node reader/retrieval-check.mjs --base-url URL [--articles 35,39] [--verify-manifest] [--timeout-ms N] [--max-bytes N] [--max-requests N] [--max-total-bytes N]');
      options[key] = key === 'articles' ? args[++i].split(',') : ['baseUrl', 'userAgent'].includes(key) ? args[++i] : Number(args[++i]);
    }
    const report = await checkRetrieval(options);
    console.log(JSON.stringify(report, null, 2));
    if (!report.ok) process.exitCode = 1;
  } catch (error) {
    console.log(JSON.stringify({ok: false, errors: [{kind: 'configuration', message: error.message}]}, null, 2));
    process.exitCode = 1;
  }
}
