const T = window.__TAURI__;
const $ = s => document.querySelector(s);
const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const GB = 1073741824, MB = 1048576;
const fmt = (b, d = 1) => b >= GB ? (b / GB).toFixed(d) + ' GB' : Math.max(0, Math.round(b / MB)) + ' MB';
const P = { home: "M3 11l9-8 9 8M5 10v10h14V10", clean: "M4 20h16M7 20l2-9h6l2 9M10 11V5a2 2 0 014 0v6", rocket: "M5 15c-1 3-1 5-1 5s2 0 5-1M14 4c3 0 6 3 6 6-2 4-5 7-9 8l-5-5c1-4 4-7 8-9zM15 9h.01", wifi: "M2 9a15 15 0 0120 0M5 12.5a10 10 0 0114 0M8.5 16a5 5 0 017 0M12 19.5h.01", power: "M12 3v9M6.3 6.3a8 8 0 1011.4 0", hist: "M3 12a9 9 0 109-9 9 9 0 00-7 3.4M3 4v4h4M12 8v4l3 2", gear: "M12 15a3 3 0 100-6 3 3 0 000 6zM12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2", chat: "M21 12a8 8 0 01-11.6 7.1L4 20l1-4.6A8 8 0 1121 12z", bolt: "M13 2L4 14h7l-1 8 9-12h-7z", chip: "M9 3v3M15 3v3M9 18v3M15 18v3M3 9h3M3 15h3M18 9h3M18 15h3M7 7h10v10H7z", gauge: "M4 18a8 8 0 1116 0M12 18l4-5", disk: "M4 7c0-2 16-2 16 0v10c0 2-16 2-16 0zM4 12c0 2 16 2 16 0" };
const ic = n => '<svg class="i" viewBox="0 0 24 24"><path d="' + P[n] + '"/></svg>';

const demo = {
  system_stats: { ram_total: 8 * GB, ram_free: 3.2 * GB, cpu_percent: 34, disk_total: 238 * GB, disk_free: 78 * GB, score: 72 },
  scan_emulators: [
    { id: 'bluestacks', name: 'BlueStacks', installed: true, running: true, found_dirs: ['C:\\ProgramData\\BlueStacks_nxt'], total_bytes: 14.2 * GB, candidates: [], breakdown: [{ path: 'C:\\ProgramData\\BlueStacks_nxt\\Engine', size_bytes: 13.5 * GB, protected: true }] },
    { id: 'ldplayer', name: 'LDPlayer', installed: true, running: false, found_dirs: [], total_bytes: 6.1 * GB, candidates: [], breakdown: [] }
  ]
};
const call = c => T ? T.core.invoke(c) : new Promise(r => setTimeout(() => r(demo[c]), 500));

let stats = null, emus = null, tm;
function toast(m) { const el = $('#ts'); el.textContent = m; el.classList.add('s'); clearTimeout(tm); tm = setTimeout(() => el.classList.remove('s'), 2800); }

function renderStats() {
  const s = stats; if (!s) return;
  $('#rp').style.strokeDashoffset = 226.2 * (1 - s.score / 100);
  $('#rn').textContent = s.score + '%';
  $('#h1').textContent = s.score >= 75 ? 'Your PC is ready to play.' : s.score >= 50 ? 'Your PC could use some attention.' : 'Your PC is under heavy load.';
  const load = s.cpu_percent < 60 ? 'Normal load' : 'High load';
  $('#tiles').innerHTML =
    '<div class="card tile"><div class="l">Free RAM' + ic('chip') + '</div><div class="v"><b>' + fmt(s.ram_free) + '</b><span>of ' + fmt(s.ram_total, 0) + '</span></div></div>' +
    '<div class="card tile"><div class="l">CPU' + ic('gauge') + '</div><div class="v"><b>' + Math.round(s.cpu_percent) + '%</b><span>' + load + '</span></div></div>' +
    '<div class="card tile"><div class="l">System drive' + ic('disk') + '</div><div class="v"><b>' + fmt(s.disk_free) + '</b><span>free of ' + fmt(s.disk_total, 0) + '</span></div></div>';
}

function renderEmus() {
  const l = $('#list');
  if (!emus) { l.innerHTML = '<div class="empty">Scanning…</div>'; return; }
  const found = emus.filter(e => e.installed);
  if (!found.length) { l.innerHTML = '<div class="empty">No supported emulator found. Supported: BlueStacks, LDPlayer, Gameloop. Install one and press Scan again.</div>'; return; }
  l.innerHTML = found.map(e => {
    const i = emus.indexOf(e);
    return '<div class="row"><div class="gr"><div class="nm">' + esc(e.name) + '</div><div class="cap">Cache cleaning arrives in a later update</div></div>' +
      '<span class="chip ' + (e.running ? 'g' : '') + '"><i></i>' + (e.running ? 'Running' : 'Idle') + '</span>' +
      '<b class="sz">' + fmt(e.total_bytes) + '</b><button class="btn sm" onclick="details(' + i + ')">Details</button></div>';
  }).join('');
}

function modal(h) { const o = $('#ov'); o.innerHTML = '<div class="md" role="dialog" aria-modal="true">' + h + '</div>'; o.classList.add('s'); }
function closeModal() { $('#ov').classList.remove('s'); }

function details(i) {
  const e = emus[i];
  const rows = e.breakdown.length ? e.breakdown.map(f => '<div class="fr"><span>' + esc(f.path) + '</span>' + (f.protected ? '<em class="chip w">Protected</em>' : '') + '<b>' + fmt(f.size_bytes, 2) + '</b></div>').join('') : '<p>No folders to show.</p>';
  const cands = e.candidates.filter(c => c.exists).map(c => '<div class="fr"><span>' + esc(c.label) + ': ' + esc(c.path) + '</span><em class="chip">Not verified</em><b>' + fmt(c.size_bytes, 2) + '</b></div>').join('');
  modal('<h3>' + esc(e.name) + ' folders</h3><p>Largest folders found. Protected folders hold your games and accounts. EmuCleaner will never touch them.</p>' + rows +
    (cands ? '<p>Possible cleanable folders (still being verified):</p>' + cands : '') +
    '<div class="ma"><button class="btn sec" onclick="closeModal()">Close</button><button class="btn pri" onclick="copyReport()">Copy scan report</button></div>');
}

async function copyReport() {
  const data = { app: 'EmuCleaner 0.1.0 stage 1', time: new Date().toISOString(), userAgent: navigator.userAgent, stats, emulators: emus };
  const text = JSON.stringify(data, null, 2);
  try { await navigator.clipboard.writeText(text); toast('Scan report copied. Paste it to the developer.'); }
  catch (e) {
    const ta = document.createElement('textarea'); ta.value = text; document.body.appendChild(ta); ta.select();
    try { document.execCommand('copy'); toast('Scan report copied. Paste it to the developer.'); } catch (_) { toast('Could not copy the report.'); }
    ta.remove();
  }
}

async function load() {
  emus = null; renderEmus();
  try { stats = await call('system_stats'); renderStats(); } catch (e) { toast('Could not read system stats.'); }
  try { emus = await call('scan_emulators'); renderEmus(); } catch (e) { emus = []; renderEmus(); toast('Scan failed.'); }
  $('#sub').textContent = 'Based on free RAM, disk space and CPU load. Checked just now.';
}

$('#lg').innerHTML = ic('bolt'); $('#set').innerHTML = ic('gear'); $('#fb').innerHTML = ic('chat');
$('#nav').innerHTML = [['home', 'Home'], ['clean', 'Clean'], ['rocket', 'Boost'], ['wifi', 'Network'], ['power', 'Startup'], ['hist', 'History']]
  .map((n, i) => '<button class="nv' + (i ? '' : ' on') + '" data-n="' + n[1] + '">' + ic(n[0]) + '<span>' + n[1] + '</span></button>').join('');
$('#nav').onclick = e => { const b = e.target.closest('.nv'); if (b && b.dataset.n !== 'Home') toast(b.dataset.n + ' arrives in a later update.'); };
$('#set').onclick = () => toast('Settings arrives in a later update.');
$('#fb').onclick = () => toast('For now, use Copy scan report and send it to the developer.');
$('#scan').onclick = () => { load(); toast('Scanning…'); };
$('#rep').onclick = copyReport;
$('#ov').onclick = e => { if (e.target.id === 'ov') closeModal(); };
document.onkeydown = e => { if (e.key === 'Escape') closeModal(); };
if (!T) $('#note').textContent = 'Preview mode: showing demo data because this page is not running inside the EmuCleaner app.';
load();
