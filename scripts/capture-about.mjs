// Regenerates the three walkthrough screenshots on /about
// (public/images/about-{new,pick,solved}.webp) from a running local
// `npx wrangler dev`. Drives headless Chrome over the DevTools protocol with
// Node built-ins only (Node 22+), then converts with cwebp. Usage:
//
//   npx wrangler dev                           # in another terminal
//   node scripts/capture-about.mjs /tmp/shots
//   for n in new pick solved; do
//     cwebp -q 82 -resize 600 0 /tmp/shots/$n.png -o public/images/about-$n.webp
//   done
//
// BASE overrides the dev server URL (default http://localhost:8787).
// Loads a fresh puzzle, fills part of it in with the solution, picks an empty
// cell, then fills the rest and presses Check ?. The toolbar's id-<hash> label
// is hidden in every shot: it changes with each code change, and a stale id in
// a screenshot would only confuse anyone comparing it with their own toolbar.
import { spawn } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const BASE = process.env.BASE || 'http://localhost:8787';
const OUT = process.argv[2];
if (!OUT) { console.error('usage: node scripts/capture-about.mjs <out-dir>'); process.exit(1); }
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const profile = mkdtempSync(join(process.env.TMPDIR || tmpdir(), 'cdp-'));
const chrome = spawn(process.env.CHROME || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', [
  '--headless=new', '--remote-debugging-port=9333', `--user-data-dir=${profile}`,
  '--hide-scrollbars', '--no-first-run', '--force-color-profile=srgb', 'about:blank',
], { stdio: 'ignore' });

let targets;
for (let i = 0; i < 50; i++) {
  try { targets = await (await fetch('http://127.0.0.1:9333/json/list')).json(); break; } catch { await sleep(200); }
}
const page = targets.find((t) => t.type === 'page');
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let id = 0;
const pending = new Map();
ws.onmessage = (e) => {
  const m = JSON.parse(e.data);
  if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); }
};
const send = (method, params = {}) => new Promise((resolve, reject) => {
  const n = ++id;
  pending.set(n, (m) => (m.error ? reject(new Error(`${method}: ${m.error.message}`)) : resolve(m.result)));
  ws.send(JSON.stringify({ id: n, method, params }));
});
const evaluate = async (expression) => {
  const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || 'evaluate failed');
  return r.result.value;
};

// The component renders into a shadow root. Everything below runs in the page.
const HELPERS = `
  const root = () => document.querySelector('proto-sudoku').shadowRoot;
  const q = (s) => [...root().querySelectorAll(s)];
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  const button = (label) => q('button').find((b) => b.textContent.trim() === label);
  const cell = (i) => root().querySelector('label.cell-' + i);
  const data = () => JSON.parse(localStorage.getItem('proto-sudoku::data'));
  const solution = () => atob(data().ref);
  async function fill(indexes) {
    const sol = solution();
    for (const i of indexes) {
      cell(i).click(); await wait(30);
      button(sol[i]).click(); await wait(30);
    }
  }
`;
const run = (body) => evaluate(`(async () => { ${HELPERS} ${body} })()`);

// Screenshot just the component, with a little margin.
// Each render replaces the toolbar, so the id label is hidden again per shot.
// visibility keeps its space, so the layout matches the live page.
async function shot(name) {
  const box = await run(`root().querySelector('.build')?.style.setProperty('visibility', 'hidden');
    const r = root().querySelector('#app').getBoundingClientRect();
    return { x: r.x, y: r.y, width: r.width, height: r.height };`);
  const pad = 16;
  const { data } = await send('Page.captureScreenshot', {
    format: 'png',
    clip: { x: Math.max(0, box.x - pad), y: Math.max(0, box.y - pad), width: box.width + pad * 2, height: box.height + pad * 2, scale: 1 },
  });
  writeFileSync(join(OUT, `${name}.png`), Buffer.from(data, 'base64'));
  console.log('wrote', name);
}

await send('Page.enable');
await send('Emulation.setDeviceMetricsOverride', { width: 420, height: 720, deviceScaleFactor: 2, mobile: false });
await send('Page.navigate', { url: BASE + '/' });
await sleep(1500);
await run(`localStorage.clear(); return true;`);
await send('Page.navigate', { url: BASE + '/' });
await sleep(1500);

// 1. A fresh puzzle.
await run(`button('New Puzzle').click(); await wait(1500); return true;`);
await shot('new');

// 2. Part-way through: about a third of the blanks filled, one empty cell picked.
const picked = await run(`
  const puzzle = data().puzzle;
  const blanks = [...puzzle].map((c, i) => (c === '.' ? i : -1)).filter((i) => i >= 0);
  const early = blanks.filter((i) => Math.floor(i / 9) < 3 || i % 9 < 2);
  await fill(early);
  const rest = blanks.filter((i) => !early.includes(i));
  const pick = rest.find((i) => Math.floor(i / 9) === 4) ?? rest[0];
  cell(pick).click(); await wait(300);
  return pick;`);
console.log('picked cell', picked);
await shot('pick');

// 3. Every blank filled with the solution, then Check ?.
await run(`
  const puzzle = data().puzzle;
  const blanks = [...puzzle].map((c, i) => (c === '.' ? i : -1)).filter((i) => i >= 0);
  const empty = blanks.filter((i) => cell(i).textContent.trim() === '');
  await fill(empty);
  button('Check ?').click(); await wait(500);
  return true;`);
await shot('solved');

ws.close();
chrome.kill();
