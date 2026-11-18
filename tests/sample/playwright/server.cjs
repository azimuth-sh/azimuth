const fs = require('node:fs');
function journal(event) {
  if (process.env.SERVER_JOURNAL) fs.appendFileSync(process.env.SERVER_JOURNAL,
    JSON.stringify({ event, pid: process.pid, time_ms: Date.now() }) + '\n');
}
journal('started');
const server = require('node:http').createServer((request, response) => response.end('library desk'));
server.listen(Number(process.env.SERVER_PORT), '127.0.0.1', () => journal('listening'));
process.on('SIGTERM', () => {
  journal('SIGTERM');
  setTimeout(() => {
    server.close(() => { journal('cleaned'); process.exit(0); });
  }, 500);
});
