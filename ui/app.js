const T = window.__TAURI__;
const $ = s => document.querySelector(s);
const esc = s => String(s == null ? '' : s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const GB = 1073741824, MB = 1048576, KB = 1024;
const fmt = (b, d = 1) => b >= GB ? (b / GB).toFixed(d) + ' GB' : b >= MB ? Math.round(b / MB) + ' MB' : Math.max(0, Math.round(b / KB)) + ' KB';
const ago = ms => { const m = Math.round((Date.now() - ms) / 60000); if (m < 1) return 'just now'; if (m < 60) return m + ' min ago'; const h = Math.round(m / 60); return h < 24 ? h + ' h ago' : Math.round(h / 24) + ' d ago'; };
const when = ms => new Date(ms).toLocaleString([], { month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit' });
const P = { home: "M3 11l9-8 9 8M5 10v10h14V10", clean: "M4 20h16M7 20l2-9h6l2 9M10 11V5a2 2 0 014 0v6", rocket: "M5 15c-1 3-1 5-1 5s2 0 5-1M14 4c3 0 6 3 6 6-2 4-5 7-9 8l-5-5c1-4 4-7 8-9zM15 9h.01", wifi: "M2 9a15 15 0 0120 0M5 12.5a10 10 0 0114 0M8.5 16a5 5 0 017 0M12 19.5h.01", power: "M12 3v9M6.3 6.3a8 8 0 1011.4 0", hist: "M3 12a9 9 0 109-9 9 9 0 00-7 3.4M3 4v4h4M12 8v4l3 2", gear: "M12 15a3 3 0 100-6 3 3 0 000 6zM12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2", chat: "M21 12a8 8 0 01-11.6 7.1L4 20l1-4.6A8 8 0 1121 12z", bolt: "M13 2L4 14h7l-1 8 9-12h-7z", chip: "M9 3v3M15 3v3M9 18v3M15 18v3M3 9h3M3 15h3M18 9h3M18 15h3M7 7h10v10H7z", gauge: "M4 18a8 8 0 1116 0M12 18l4-5", disk: "M4 7c0-2 16-2 16 0v10c0 2-16 2-16 0zM4 12c0 2 16 2 16 0" };
const ic = n => '<svg class="i" viewBox="0 0 24 24"><path d="' + P[n] + '"/></svg>';

// Demo data so the interface can be previewed in a normal browser.
const demo = {
  system_stats: { ram_total: 8 * GB, ram_free: 3.2 * GB, cpu_percent: 34, disk_total: 238 * GB, disk_free: 78 * GB, score: 72 },
  scan_emulators: [{ id: 'bluestacks', name: 'BlueStacks', installed: true, running: true, found_dirs: ['C:\\ProgramData\\BlueStacks_nxt'], total_bytes: 14.2 * GB, candidates: [], breakdown: [{ path: 'C:\\ProgramData\\BlueStacks_nxt\\Engine', size_bytes: 13.5 * GB, protected: true }] }],
  scan_junk: [
    { id: 'win_temp', label: 'Temporary files', group: 'Windows', description: 'Files apps left in your Temp folder', size_bytes: 1.2 * GB, files: 3400, permanent: false, default_on: true, warn_running: false },
    { id: 'crash_dumps', label: 'Crash dumps', group: 'Windows', description: 'Memory dumps from crashed apps', size_bytes: 300 * MB, files: 12, permanent: false, default_on: true, warn_running: false },
    { id: 'recycle', label: 'Recycle Bin', group: 'Windows', description: 'Files you already deleted. Emptying it cannot be undone', size_bytes: 600 * MB, files: 0, permanent: true, default_on: false, warn_running: false }],
  clean_selected: { batch_id: '1', moved_bytes: 1.5 * GB, moved_files: 3412, skipped_files: 4, permanent_bytes: 0, errors: [] },
  quarantine_list: [{ id: '1', created_ms: Date.now() - 3600000, label: 'Junk cleanup', total_bytes: 1.1 * GB, files: 120, days_left: 7 }],
  quarantine_restore: { restored: 120, skipped: 0 },
  quarantine_empty: 0,
  history_list: [{ ts_ms: Date.now() - 3600000, kind: 'clean', text: '120 files cleaned', bytes: 1.1 * GB }],
  boost_candidates: [{ name: 'onedrive.exe', display: 'OneDrive', category: 'Cloud sync', count: 2, default_on: true }, { name: 'teams.exe', display: 'Microsoft Teams', category: 'Chat', count: 4, default_on: false }],
  boost_status: { active: false, messages: [], closed: [] },
  boost_start: { active: true, started_ms: Date.now(), messages: ['High Performance power plan is on.', 'Closed 1 background program(s).'], closed: [{ name: 'onedrive.exe', path: '' }] },
  boost_stop: { active: false, messages: ['Previous power plan restored.'], closed: [] },
  boost_tick: { active: true, messages: [], closed: [] },
  startup_list: [
    { id: 'HKCU Run|Discord', name: 'Discord', command: 'Update.exe', source: 'HKCU Run', enabled: true, impact: 'High', keep: false },
    { id: 'HKLM Run|RtkAudUService', name: 'Realtek Audio', command: 'RtkAudUService64.exe', source: 'HKLM Run', enabled: true, impact: 'Unknown', keep: true }],
  startup_set: null,
  lag_check: { target: '1.1.1.1:443', ping_ms: 38, jitter_ms: 6, loss_pct: 0, rating: 'Good', message: 'Your connection looks stable for gaming. Measured to a public server, so your game server may differ.', series: [38, 40, 36, 39, 41, 37, 38, 36, 40, 39, 38, 37, 41, 38, 36, 39, 40, 38, 37, 39] },
  get_settings: { retention_days: 7, min_age_hours: 24, custom_close_apps: [], feedback_url: '' },
  save_settings: { retention_days: 7, min_age_hours: 24, custom_close_apps: [], feedback_url: '' }
};
const inv = (c, a) => T ? T.core.invoke(c, a || {}) : new Promise(r => setTimeout(() => r(JSON.parse(JSON.stringify(demo[c] === undefined ? null : demo[c]))), 250));

const S = { page: 'home', stats: null, emus: null, hist: null, q: null, boost: null, cands: null, bo: { power: true, prio: true, apps: new Set(), init: false }, junk: null, sel: new Set(), startup: null, sf: 'all', lag: null, lagBusy: false, settings: { retention_days: 7, min_age_hours: 24, custom_close_apps: [], feedback_url: '' } };
let tm, ticker;
function toast(m) { const el = $('#ts'); el.textContent = m; el.classList.add('s'); clearTimeout(tm); tm = setTimeout(() => el.classList.remove('s'), 3000); }
function modal(h) { const o = $('#ov'); o.innerHTML = '<div class="md" role="dialog" aria-modal="true">' + h + '</div>'; o.classList.add('s'); }
function closeModal() { $('#ov').classList.remove('s'); }
function confirmBox(title, html, okLabel, fn) { window._ok = fn; modal('<h3>' + esc(title) + '</h3>' + html + '<div class="ma"><button class="btn sec" onclick="closeModal()">Cancel</button><button class="btn pri" onclick="closeModal();_ok()">' + esc(okLabel) + '</button></div>'); }
async function copyText(text) {
  try { await navigator.clipboard.writeText(text); return true; } catch (e) {
    const ta = document.createElement('textarea'); ta.value = text; document.body.appendChild(ta); ta.select();
    let ok = false; try { ok = document.execCommand('copy'); } catch (_) { } ta.remove(); return ok;
  }
}
const msgOf = e => String(e && e.message ? e.message : e);

const NAV = [['home', 'Home', 'home'], ['clean', 'Clean', 'clean'], ['boost', 'Boost', 'rocket'], ['net', 'Network', 'wifi'], ['start', 'Startup', 'power'], ['hist', 'History', 'hist']];
function buildNav() { $('#nav').innerHTML = NAV.map(n => '<button class="nv' + (S.page === n[0] ? ' on' : '') + '" onclick="go(\'' + n[0] + '\')">' + ic(n[2]) + '<span>' + n[1] + '</span></button>').join(''); }
function updateStatus() {
  const on = S.boost && S.boost.active;
  $('#bs').textContent = on ? 'On' : 'Off'; $('#bdot').classList.toggle('g', !!on);
  $('#qs').textContent = fmt((S.q || []).reduce((a, b) => a + b.total_bytes, 0));
}
function render() { const v = VIEW[S.page] || VIEW.home; $('#view').innerHTML = v(); updateStatus(); }
function go(p) { S.page = p; buildNav(); render(); (LOAD[p] || (() => { }))(); }
const loading = t => '<div class="card ld">' + esc(t) + '</div>';
const still = p => S.page === p;

// ---------- Home ----------
const VIEW = {};
VIEW.home = () => {
  const s = S.stats, on = S.boost && S.boost.active;
  const score = s ? s.score : null;
  const title = on ? 'Boost is on. Have a good game.' : score == null ? 'Checking your PC…' : score >= 75 ? 'Your PC is ready to play.' : score >= 50 ? 'Your PC could use some attention.' : 'Your PC is under heavy load.';
  const emus = (S.emus || []).filter(e => e.installed);
  const rows = S.emus == null ? '<div class="empty">Scanning…</div>' : !emus.length ? '<div class="empty">No supported emulator found. Supported: BlueStacks, LDPlayer, Gameloop.</div>' :
    emus.map(e => { const i = S.emus.indexOf(e); return '<div class="row"><div class="gr"><div class="nm">' + esc(e.name) + '</div></div><span class="chip ' + (e.running ? 'g' : '') + '"><i></i>' + (e.running ? 'Running' : 'Idle') + '</span><b class="sz">' + fmt(e.total_bytes) + '</b><button class="btn sm" onclick="details(' + i + ')">Details</button><button class="btn sm" onclick="go(\'clean\')">Clean</button></div>'; }).join('');
  const lastClean = (S.hist || []).find(e => e.kind === 'clean');
  const tiles = s ? '<div class="card tile"><div class="l">Free RAM' + ic('chip') + '</div><div class="v"><b>' + fmt(s.ram_free) + '</b><span>of ' + fmt(s.ram_total, 0) + '</span></div></div>' +
    '<div class="card tile"><div class="l">CPU' + ic('gauge') + '</div><div class="v"><b>' + Math.round(s.cpu_percent) + '%</b><span>' + (s.cpu_percent < 60 ? 'Normal load' : 'High load') + '</span></div></div>' +
    '<div class="card tile"><div class="l">System drive' + ic('disk') + '</div><div class="v"><b>' + fmt(s.disk_free) + '</b><span>free of ' + fmt(s.disk_total, 0) + '</span></div></div>' : '';
  return '<section class="card hero"><div class="ring"><svg viewBox="0 0 84 84"><circle cx="42" cy="42" r="36" stroke="var(--bd)"/><circle id="rp" cx="42" cy="42" r="36" style="stroke-dashoffset:' + (226.2 * (1 - (score || 0) / 100)) + '"/></svg><div class="n">' + (score == null ? '--' : score + '%') + '</div></div>' +
    '<div class="ht"><h1>' + title + '</h1><p>Based on free RAM, disk space and CPU load.</p></div>' +
    '<div class="ha"><button class="btn pri' + (on ? ' stop' : '') + '" onclick="toggleBoost()">' + ic('rocket') + (on ? 'Stop &amp; Restore' : 'Boost &amp; Play') + '</button><button class="btn sec" onclick="quickClean()">Quick Clean</button></div></section>' +
    '<div class="sh"><h2>Detected emulators</h2><p>Your game data and accounts are never touched.</p><button class="lk" onclick="copyReport()">Copy scan report</button><button class="lk" onclick="LOAD.home()">Scan again</button></div>' +
    '<section class="card">' + rows + '</section><section class="tiles">' + tiles + '</section>' +
    '<p class="note">' + (lastClean ? 'Last cleanup: ' + fmt(lastClean.bytes) + ' freed, ' + ago(lastClean.ts_ms) + '.' : 'No cleanups yet.') + (T ? '' : ' Preview mode: demo data.') + '</p>';
};
async function safe(fn, what) { try { return await fn(); } catch (e) { toast(what + ': ' + msgOf(e)); return null; } }
const LOAD = {};
LOAD.home = async () => {
  S.emus = null; if (still('home')) render();
  const [st, bo, hi, q] = await Promise.all([safe(() => inv('system_stats'), 'Stats'), safe(() => inv('boost_status'), 'Boost'), safe(() => inv('history_list'), 'History'), safe(() => inv('quarantine_list'), 'Quarantine')]);
  S.stats = st; S.boost = bo; S.hist = hi || []; S.q = q || []; if (bo && bo.active) startTicker(); if (still('home')) render();
  S.emus = (await safe(() => inv('scan_emulators'), 'Scan')) || []; if (still('home')) render();
};
function details(i) {
  const e = S.emus[i];
  const rows = e.breakdown.length ? e.breakdown.map(f => '<div class="fr"><span>' + esc(f.path) + '</span>' + (f.protected ? '<em class="chip w">Protected</em>' : '') + '<b>' + fmt(f.size_bytes, 2) + '</b></div>').join('') : '<p>No folders to show.</p>';
  modal('<h3>' + esc(e.name) + ' folders</h3><p>Largest folders found. Protected folders hold your games and accounts. EmuCleaner never touches them.</p>' + rows + '<div class="ma"><button class="btn sec" onclick="closeModal()">Close</button><button class="btn pri" onclick="copyReport()">Copy scan report</button></div>');
}
async function copyReport() {
  const data = { app: 'EmuCleaner 0.2.0', time: new Date().toISOString(), userAgent: navigator.userAgent, stats: S.stats, emulators: S.emus };
  toast((await copyText(JSON.stringify(data, null, 2))) ? 'Scan report copied. Paste it to the developer.' : 'Could not copy the report.');
}
async function quickClean() {
  const items = await safe(() => inv('scan_junk'), 'Scan'); if (!items) return;
  S.junk = items;
  const pick = items.filter(i => i.default_on && !i.permanent), total = pick.reduce((a, i) => a + i.size_bytes, 0);
  if (!pick.length) { toast('Nothing to clean right now.'); return; }
  confirmBox('Quick Clean', '<p>' + fmt(total) + ' of junk files will be moved to Quarantine. You can restore them from History for ' + S.settings.retention_days + ' days.</p>', 'Clean now', () => runClean(pick.map(i => i.id)));
}
async function runClean(ids) {
  modal('<h3>Cleaning…</h3><p>Please wait. Files that are in use are skipped.</p>');
  const r = await safe(() => inv('clean_selected', { ids }), 'Clean');
  if (!r) { closeModal(); return; }
  modal('<h3>Done</h3><p>' + fmt(r.moved_bytes) + ' in ' + r.moved_files + ' file(s) moved to Quarantine.' + (r.skipped_files ? ' ' + r.skipped_files + ' file(s) were in use and skipped.' : '') + (r.permanent_bytes ? ' ' + fmt(r.permanent_bytes) + ' removed permanently.' : '') + ' You can restore them from History.</p>' +
    (r.errors.length ? '<div class="err">' + r.errors.map(esc).join('<br>') + '</div>' : '') + '<div class="ma"><button class="btn pri" onclick="closeModal()">OK</button></div>');
  S.junk = null; S.hist = (await safe(() => inv('history_list'), 'History')) || S.hist; S.q = (await safe(() => inv('quarantine_list'), 'Quarantine')) || S.q;
  if (still('clean')) LOAD.clean(); else if (still('home')) { S.stats = await safe(() => inv('system_stats'), 'Stats') || S.stats; render(); } else render();
}

// ---------- Clean ----------
VIEW.clean = () => {
  if (!S.junk) return '<div class="pg"><h1>Clean</h1></div>' + loading('Scanning for junk files…');
  const items = S.junk;
  const groups = [...new Set(items.map(i => i.group))];
  const sel = items.filter(i => S.sel.has(i.id)), total = sel.reduce((a, i) => a + i.size_bytes, 0), perm = sel.some(i => i.permanent);
  const list = !items.length ? '<div class="empty">Nothing to clean. Your PC is tidy.</div>' : groups.map(g => '<div class="gh">' + esc(g) + '</div>' + items.filter(i => i.group === g).map(it => {
    const i = items.indexOf(it);
    return '<label class="ri"><input type="checkbox" ' + (S.sel.has(it.id) ? 'checked' : '') + ' onchange="selToggle(' + i + ')"><div class="gr"><div class="nm">' + esc(it.label) + '</div><div class="cap">' + esc(it.description) + (it.files ? ' (' + it.files + ' files)' : '') + '</div></div>' +
      (it.warn_running ? '<em class="chip w">Emulator running</em>' : '') + (it.permanent ? '<em class="chip r">Cannot be undone</em>' : '<em class="chip g"><i></i>Undoable</em>') + '<b class="sz">' + fmt(it.size_bytes) + '</b></label>';
  }).join('')).join('');
  return '<div class="pg"><h1>Clean</h1><p>Review what will be cleaned. Nothing is deleted straight away.</p></div><div class="two"><section class="card">' + list + '</section>' +
    '<section class="card sum"><span class="cap">Selected</span><div class="big">' + fmt(total) + '</div><p>Cleaned files move to Quarantine for ' + S.settings.retention_days + ' days. Game data, accounts and settings are never touched. Files newer than ' + S.settings.min_age_hours + ' hours are skipped.</p>' +
    (perm ? '<div class="bn">The Recycle Bin is emptied for good and cannot be restored.</div>' : '') +
    '<button class="btn pri" ' + (sel.length ? '' : 'disabled') + ' onclick="cleanSelected()">Clean selected</button><button class="btn sec" onclick="LOAD.clean()">Scan again</button></section></div>';
};
LOAD.clean = async () => { S.junk = null; if (still('clean')) render(); const r = await safe(() => inv('scan_junk'), 'Scan'); S.junk = r || []; S.sel = new Set(S.junk.filter(i => i.default_on).map(i => i.id)); if (still('clean')) render(); };
function selToggle(i) { const id = S.junk[i].id; S.sel.has(id) ? S.sel.delete(id) : S.sel.add(id); render(); }
function cleanSelected() {
  const ids = [...S.sel], perm = S.junk.some(i => S.sel.has(i.id) && i.permanent);
  const total = S.junk.filter(i => S.sel.has(i.id)).reduce((a, i) => a + i.size_bytes, 0);
  confirmBox('Clean selected items', '<p>' + fmt(total) + ' will be cleaned.' + (perm ? ' The Recycle Bin will be emptied permanently.' : ' Everything can be restored from History.') + '</p>', 'Clean now', () => runClean(ids));
}

// ---------- Boost ----------
VIEW.boost = () => {
  if (!S.boost || !S.cands) return '<div class="pg"><h1>Boost &amp; Play</h1></div>' + loading('Checking running programs…');
  const b = S.boost, on = b.active, o = S.bo;
  const apps = !S.cands.length ? '<div class="empty">No closable background programs are running.</div>' : S.cands.map((c, i) => '<label class="ri"><input type="checkbox" ' + (o.apps.has(c.name) ? 'checked' : '') + (on ? ' disabled' : '') + ' onchange="boostApp(' + i + ')"><div class="gr"><div class="nm">' + esc(c.display) + '</div><div class="cap">' + esc(c.category) + ', ' + c.count + ' running</div></div></label>').join('');
  const sw = (key, label, desc) => '<div class="ri"><div class="gr"><div class="nm">' + label + '</div><div class="cap">' + desc + '</div></div><label class="sw"><input type="checkbox" ' + (o[key] ? 'checked' : '') + (on ? ' disabled' : '') + ' onchange="S.bo.' + key + '=this.checked"><span></span></label></div>';
  return '<div class="pg"><h1>Boost &amp; Play</h1><p>Get your PC ready for gaming. Everything is restored when you stop.</p></div><div class="two"><div style="display:grid;gap:16px"><section class="card">' +
    sw('power', 'High Performance power plan', 'Uses full power while you play') + sw('prio', 'Raise emulator priority', 'Gives running emulators more CPU time') + '</section>' +
    '<section class="card"><div class="gh">Close these background programs</div>' + apps + '</section>' +
    '<p class="note">Programs closed by Boost are reopened when you stop. Security software, audio and graphics drivers, Discord, OBS and emulators are never closed.</p></div>' +
    '<section class="card sum"><span class="cap">Status</span><div class="big">' + (on ? 'Boost is on' : 'Not active') + '</div>' + (on ? '<p>Started ' + ago(b.started_ms) + '.</p>' : '') +
    (b.messages && b.messages.length ? '<p>' + b.messages.map(esc).join('<br>') + '</p>' : '') +
    '<button class="btn pri' + (on ? ' stop' : '') + '" onclick="toggleBoost()">' + ic('rocket') + (on ? 'Stop &amp; Restore' : 'Start Boost') + '</button></section></div>';
};
LOAD.boost = async () => {
  S.cands = null; if (still('boost')) render();
  const [b, c] = await Promise.all([safe(() => inv('boost_status'), 'Boost'), safe(() => inv('boost_candidates'), 'Programs')]);
  S.boost = b || { active: false, messages: [] }; S.cands = c || [];
  if (!S.bo.init) { S.bo.apps = new Set(S.cands.filter(x => x.default_on).map(x => x.name)); S.bo.init = true; }
  if (still('boost')) render();
};
function boostApp(i) { const n = S.cands[i].name; S.bo.apps.has(n) ? S.bo.apps.delete(n) : S.bo.apps.add(n); }
async function toggleBoost() {
  const on = S.boost && S.boost.active;
  if (on) { const r = await safe(() => inv('boost_stop'), 'Boost'); if (r) { S.boost = r; stopTicker(); toast('Boost stopped. Your settings are restored.'); } }
  else {
    if (!S.cands) S.cands = (await safe(() => inv('boost_candidates'), 'Programs')) || [];
    if (!S.bo.init) { S.bo.apps = new Set(S.cands.filter(x => x.default_on).map(x => x.name)); S.bo.init = true; }
    const r = await safe(() => inv('boost_start', { powerPlan: S.bo.power, priority: S.bo.prio, closeApps: [...S.bo.apps] }), 'Boost');
    if (r) { S.boost = r; startTicker(); toast('Boost started.'); }
  }
  render();
}
function startTicker() { if (!ticker) ticker = setInterval(async () => { const b = await safe(() => inv('boost_tick'), 'Boost'); if (b) S.boost = b; }, 10000); }
function stopTicker() { clearInterval(ticker); ticker = null; }

// ---------- Network ----------
VIEW.net = () => {
  const r = S.lag;
  const spark = r ? (() => { const v = r.series.map(x => x == null ? null : x), mx = Math.max(...v.filter(x => x != null), 1); return '<svg viewBox="0 0 300 60" preserveAspectRatio="none" width="100%" height="70"><polyline fill="none" stroke="var(--ac)" stroke-width="2" points="' + v.map((x, i) => x == null ? '' : (i * 300 / Math.max(v.length - 1, 1)).toFixed(1) + ',' + (58 - x / mx * 52).toFixed(1)).filter(Boolean).join(' ') + '"/></svg>'; })() : '';
  return '<div class="pg"><h1>Network</h1><p>Check how stable your connection is for gaming.</p></div><section class="card pad" style="display:grid;gap:14px"><div style="display:flex;align-items:center;gap:16px;flex-wrap:wrap"><div class="rate ' + (r ? r.rating : '') + '">' + (r ? r.rating : 'Not tested') + '</div><div class="gr"><p>' + (r ? esc(r.message) : 'Run a quick test to see your ping, jitter and packet loss.') + '</p></div><button class="btn pri" ' + (S.lagBusy ? 'disabled' : '') + ' onclick="runLag()">' + (S.lagBusy ? 'Testing…' : 'Run Lag Check') + '</button></div>' +
    (r ? '<div class="mets"><div class="met"><span>Ping</span><b>' + r.ping_ms + ' ms</b></div><div class="met"><span>Jitter</span><b>' + r.jitter_ms + ' ms</b></div><div class="met"><span>Packet loss</span><b>' + r.loss_pct + '%</b></div></div>' + spark : '') + '</section>' +
    '<section class="card pad"><div class="nm">Quick tips</div><p>Use a wired connection if you can.<br>Close apps that upload or download while you play.<br>Move closer to your router on Wi-Fi.</p></section><p class="note">More network tools are coming in a later update.</p>';
};
LOAD.net = () => { };
async function runLag() { S.lagBusy = true; render(); const r = await safe(() => inv('lag_check'), 'Lag check'); S.lagBusy = false; if (r) S.lag = r; render(); }

// ---------- Startup ----------
VIEW.start = () => {
  if (!S.startup) return '<div class="pg"><h1>Startup</h1></div>' + loading('Reading startup programs…');
  const all = S.startup, high = all.filter(i => i.impact === 'High' && i.enabled).length;
  const list = all.filter(i => S.sf === 'all' || (S.sf === 'high' && i.impact === 'High') || (S.sf === 'on' && i.enabled));
  const rows = !list.length ? '<div class="empty">Nothing to show.</div>' : list.map(it => { const i = all.indexOf(it); return '<div class="ri"><div class="gr"><div class="nm">' + esc(it.name) + '</div><div class="cap">' + esc(it.source) + '</div></div>' +
    (it.keep ? '<em class="chip">Recommended to keep</em>' : '') + (it.impact !== 'Unknown' ? '<em class="chip ' + (it.impact === 'High' ? 'w' : '') + '">' + it.impact + ' impact</em>' : '') +
    '<label class="sw"><input type="checkbox" ' + (it.enabled ? 'checked' : '') + (it.keep ? ' disabled' : '') + ' onchange="startupSet(' + i + ',this.checked)"><span></span></label></div>'; }).join('');
  const tab = (k, l) => '<button class="tab' + (S.sf === k ? ' on' : '') + '" onclick="S.sf=\'' + k + '\';render()">' + l + '</button>';
  return '<div class="pg"><h1>Startup</h1><p>Fewer startup apps means a faster boot.</p></div><section class="card pad"><div class="nm">' + all.length + ' startup items, ' + high + ' with high typical impact</div><p>Impact is a typical estimate based on the program, not a measurement. Turning an item off does not uninstall it.</p></section>' +
    '<div class="tabs">' + tab('all', 'All') + tab('high', 'High impact') + tab('on', 'Enabled') + '</div><section class="card">' + rows + '</section>';
};
LOAD.start = async () => { S.startup = null; if (still('start')) render(); S.startup = (await safe(() => inv('startup_list'), 'Startup')) || []; if (still('start')) render(); };
async function startupSet(i, on) {
  const it = S.startup[i];
  const r = await safe(() => inv('startup_set', { id: it.id, enabled: on }), 'Startup'); 
  const fresh = await safe(() => inv('startup_list'), 'Startup'); if (fresh) S.startup = fresh;
  if (r === null && fresh) { const now = fresh.find(x => x.id === it.id); if (now && now.enabled !== on) toast('Windows did not accept the change. Try running EmuCleaner as administrator.'); }
  render();
}

// ---------- History ----------
VIEW.hist = () => {
  if (!S.q || !S.hist) return '<div class="pg"><h1>History</h1></div>' + loading('Loading…');
  const total = S.q.reduce((a, b) => a + b.total_bytes, 0);
  const batches = !S.q.length ? '<div class="empty">Nothing in Quarantine.</div>' : S.q.map((b, i) => '<div class="ri"><div class="gr"><div class="nm">' + esc(b.label) + '</div><div class="cap">' + when(b.created_ms) + ', ' + b.files + ' files, ' + b.days_left + ' day(s) left</div></div><b class="sz">' + fmt(b.total_bytes) + '</b><button class="btn sm" onclick="restoreBatch(' + i + ')">Restore</button></div>').join('');
  const events = !S.hist.length ? '<div class="empty">No activity yet.</div>' : S.hist.map(e => '<div class="ev"><span>' + esc(e.text) + '</span><em>' + (e.bytes ? fmt(e.bytes) + ', ' : '') + when(e.ts_ms) + '</em></div>').join('');
  return '<div class="pg"><h1>History</h1><p>Cleaned files stay in Quarantine for ' + S.settings.retention_days + ' days. You can restore them.</p></div>' +
    '<section class="card pad" style="display:flex;align-items:center;gap:16px;flex-wrap:wrap"><div class="gr"><span class="cap">In Quarantine</span><div class="big" style="font-size:24px;font-weight:700">' + fmt(total) + '</div></div><button class="btn sec" ' + (total ? '' : 'disabled') + ' onclick="emptyQ()">Empty Quarantine</button></section>' +
    '<section class="card"><div class="gh">Restorable cleanups</div>' + batches + '</section><section class="card"><div class="gh">Recent activity</div>' + events + '</section>';
};
LOAD.hist = async () => { const [q, h] = await Promise.all([safe(() => inv('quarantine_list'), 'Quarantine'), safe(() => inv('history_list'), 'History')]); S.q = q || []; S.hist = h || []; if (still('hist')) render(); };
async function restoreBatch(i) {
  const b = S.q[i]; const r = await safe(() => inv('quarantine_restore', { id: b.id }), 'Restore'); if (!r) return;
  toast(r.restored + ' file(s) restored' + (r.skipped ? ', ' + r.skipped + ' skipped (already exist)' : '') + '.'); LOAD.hist();
}
function emptyQ() { confirmBox('Empty Quarantine', '<p>All cleaned files are deleted for good. This cannot be undone.</p>', 'Delete for good', async () => { await safe(() => inv('quarantine_empty'), 'Quarantine'); toast('Quarantine emptied.'); LOAD.hist(); }); }

// ---------- Settings ----------
function openSettings() {
  const s = S.settings;
  modal('<h3>Settings</h3><label class="f">Keep cleaned files restorable for (days)<input type="number" id="s_ret" min="1" max="60" value="' + s.retention_days + '"></label>' +
    '<label class="f">Never clean files newer than (hours)<input type="number" id="s_age" min="0" max="720" value="' + s.min_age_hours + '"></label>' +
    '<label class="f">Extra programs Boost may close (one per line, for example myapp.exe)<textarea id="s_apps">' + esc(s.custom_close_apps.join('\n')) + '</textarea></label>' +
    '<label class="f">Feedback web address (Google Apps Script URL)<input type="text" id="s_url" value="' + esc(s.feedback_url) + '"></label>' +
    '<div class="ma"><button class="btn sec" onclick="closeModal()">Cancel</button><button class="btn pri" onclick="saveSettings()">Save</button></div>');
}
async function saveSettings() {
  const v = { retention_days: +$('#s_ret').value || 7, min_age_hours: Math.max(0, +$('#s_age').value || 0), custom_close_apps: $('#s_apps').value.split('\n').map(x => x.trim()).filter(Boolean), feedback_url: $('#s_url').value.trim() };
  const r = await safe(() => inv('save_settings', { value: v }), 'Settings'); if (!r) return;
  S.settings = r; closeModal(); toast('Settings saved.'); S.cands = null; S.junk = null; render();
}

// ---------- Feedback ----------
function openFeedback() {
  modal('<h3>Help us improve EmuCleaner</h3><p>Tell us what is not working and what you want next.</p>' +
    '<label class="f">Type<select id="f_type"><option>Feature idea</option><option>Something is broken</option><option>Something is confusing</option></select></label>' +
    '<label class="f">What is confusing or not working?<textarea id="f_a"></textarea></label><label class="f">Which feature should we add next?<textarea id="f_b"></textarea></label>' +
    '<label class="f">Discord or email (optional)<input type="text" id="f_c"></label><label class="ck"><input type="checkbox" id="f_d"> Include basic diagnostics (RAM, CPU, installed emulators)</label>' +
    '<p>We never send your files or personal data.</p><div class="ma"><button class="btn sec" onclick="closeModal()">Cancel</button><button class="btn pri" onclick="sendFeedback()">Send</button></div>');
}
async function sendFeedback() {
  const p = { time: new Date().toISOString(), app: 'EmuCleaner 0.2.0', type: $('#f_type').value, problem: $('#f_a').value.trim(), wish: $('#f_b').value.trim(), contact: $('#f_c').value.trim() };
  if (!p.problem && !p.wish) { toast('Please write something first.'); return; }
  if ($('#f_d').checked) p.diagnostics = { stats: S.stats, emulators: (S.emus || []).map(e => ({ name: e.name, installed: e.installed, total_bytes: e.total_bytes })) };
  const url = S.settings.feedback_url;
  if (url) {
    try { await fetch(url, { method: 'POST', mode: 'no-cors', headers: { 'Content-Type': 'text/plain;charset=utf-8' }, body: JSON.stringify(p) }); closeModal(); toast('Thank you! Your feedback was sent.'); }
    catch (e) { toast('Could not send. Check your internet connection.'); }
  } else { closeModal(); toast((await copyText(JSON.stringify(p, null, 2))) ? 'No feedback address set. Your feedback was copied, please send it to the developer.' : 'No feedback address is set in Settings.'); }
}

// ---------- start ----------
$('#lg').innerHTML = ic('bolt'); $('#set').innerHTML = ic('gear'); $('#fb').innerHTML = ic('chat');
$('#set').onclick = openSettings; $('#fb').onclick = openFeedback;
$('#ov').onclick = e => { if (e.target.id === 'ov') closeModal(); };
document.onkeydown = e => { if (e.key === 'Escape') closeModal(); };
(async () => { const s = await safe(() => inv('get_settings'), 'Settings'); if (s) S.settings = s; go('home'); })();
