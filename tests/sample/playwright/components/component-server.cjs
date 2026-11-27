'use strict';
// A component of the payouts product served in its own process. It answers its report path the way
// REPORT_MODE says and writes what it saw to SUITE_JOURNAL, one JSON object per line, so a test
// can tell in what order the suite's processes did their work.
const fs = require('node:fs');
const http = require('node:http');

const name = process.env.REPORT_NAME;
const place = process.env.REPORT_PATH;
const mode = process.env.REPORT_MODE || 'ok';
const host = process.env.SERVER_HOST || '127.0.0.1';
const parent = process.ppid;

function journal(event, more) {
  if (process.env.SUITE_JOURNAL) {
    fs.appendFileSync(process.env.SUITE_JOURNAL, JSON.stringify({ source: 'server', event, name, ...more }) + '\n');
  }
}

let reads = 0;
const server = http.createServer((request, response) => {
  const path = request.url.split('?')[0];
  if (path !== place) {
    response.end('payouts component');
    return;
  }
  reads += 1;
  const body = JSON.stringify({ component: name, applied: JSON.parse(process.env.AZIMUTH_CONTEXT || '{}'), count: reads });
  journal('read', { method: request.method, host: request.headers.host, path: request.url, count: reads, mode, body: mode === 'ok' ? body : undefined });
  if (mode === 'status') {
    response.statusCode = 503;
    response.end('{"unavailable":true}');
  } else if (mode === 'not-json') {
    response.end('<html>not a report</html>');
  } else if (mode === 'closed') {
    request.socket.destroy();
  } else if (mode === 'silent') {
    // No answer: the reader gives up by itself.
  } else {
    response.setHeader('content-type', 'application/json');
    response.end(body);
  }
});

server.listen(Number(process.env.SERVER_PORT || 0), host, () => {
  const port = server.address().port;
  journal('listening', { port });
  console.log('COMPONENT_READY ' + port);
});

process.on('SIGTERM', () => {
  journal('stopped', {});
  process.exit(0);
});

// A runner that is killed leaves its servers without a parent; they must not outlive the test.
setInterval(() => {
  if (process.ppid !== parent) process.exit(0);
}, 500);
setTimeout(() => process.exit(0), 120000);
