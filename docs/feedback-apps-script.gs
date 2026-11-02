// Google Sheet feedback receiver for EmuCleaner.
// 1. Create a Google Sheet. Open Extensions > Apps Script and paste this file.
// 2. Deploy > New deployment > type "Web app". Execute as: Me. Who has access: Anyone.
// 3. Copy the web app URL and paste it into EmuCleaner > Settings > Feedback web address.
function doPost(e) {
  var ss = SpreadsheetApp.getActiveSpreadsheet();
  var sheet = ss.getSheetByName('Feedback') || ss.insertSheet('Feedback');
  var d = {};
  try { d = JSON.parse(e.postData.contents); } catch (err) { return ContentService.createTextOutput('bad request'); }
  if (sheet.getLastRow() === 0) {
    sheet.appendRow(['Time', 'App', 'Type', 'Problem', 'Wish', 'Contact', 'Diagnostics']);
  }
  var cut = function (v, n) { return String(v || '').slice(0, n); };
  sheet.appendRow([cut(d.time, 40), cut(d.app, 40), cut(d.type, 40), cut(d.problem, 2000), cut(d.wish, 2000), cut(d.contact, 200),
    d.diagnostics ? cut(JSON.stringify(d.diagnostics), 5000) : '']);
  return ContentService.createTextOutput('ok');
}
