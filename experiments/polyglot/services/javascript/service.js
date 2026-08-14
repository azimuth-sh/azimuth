const http = require('node:http');
const { realizes } = require('../../../../packages/typescript/dist/index.js');

function identity() {
  realizes('polyglot/identity', 'javascript-identifies');
  return 'javascript';
}

if (require.main === module) {
  const port = Number(process.env.PORT ?? 8085);
  http.createServer((request, response) => {
    if (request.url !== '/identity') {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200, { 'content-type': 'text/plain' });
    response.end(`${identity()}\n`);
  }).listen(port, '127.0.0.1');
}

module.exports = { identity };
