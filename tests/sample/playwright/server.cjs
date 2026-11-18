const fs = require('node:fs');
function journal(event) {
  if (process.env.SERVER_JOURNAL) fs.appendFileSync(process.env.SERVER_JOURNAL,
    JSON.stringify({ event, pid: process.pid, time_ms: Date.now(), port: serverPort() }) + '\n');
}
function serverPort() { return !server ? undefined : server.address()?.port; }
journal('started');
var server = require('node:http').createServer((request, response) => response.end('library desk'));
server.listen(Number(process.env.SERVER_PORT || 0), '127.0.0.1', () => {
  journal('listening');
  console.log('DESK_READY ' + server.address().port);
});
process.on('exit', () => journal('exited'));
process.on('SIGTERM', () => {
  journal('SIGTERM');
  setTimeout(() => {
    server.close(() => { journal('cleaned'); process.exit(0); });
  }, 500);
});
