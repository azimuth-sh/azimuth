'use strict';

const fs = require('node:fs');
const http = require('node:http');
const https = require('node:https');
const net = require('node:net');
const path = require('node:path');

const READ_LIMIT_MS = 10000;
const LOOK_LIMIT_MS = 3000;
const PLAN = Symbol.for('azimuth-playwright.plan');

// The address of an entry is its origin, so a path or a query in `url` never reaches a read.
function addressOf(entry) {
  if (entry.url) return new URL(entry.url).origin;
  if (entry.port) return `http://localhost:${entry.port}`;
  return undefined;
}

function acceptsConnection(host, port) {
  return new Promise(resolve => {
    const socket = net.connect({ host, port });
    const finish = answered => {
      socket.destroy();
      resolve(answered);
    };
    socket.setTimeout(LOOK_LIMIT_MS, () => finish(false));
    socket.on('connect', () => finish(true));
    socket.on('error', () => finish(false));
  });
}

// `localhost` names two addresses, and a server may hold only one of them.
async function answers(address) {
  const url = new URL(address);
  const port = Number(url.port) || (url.protocol === 'https:' ? 443 : 80);
  const host = url.hostname.replace(/^\[|\]$/g, '');
  const hosts = host === 'localhost' ? ['127.0.0.1', '::1'] : [host];
  const results = await Promise.all(hosts.map(candidate => acceptsConnection(candidate, port)));
  return results.includes(true);
}

function read(address, report, phase) {
  return new Promise(resolve => {
    const failed = error => resolve({ phase, error });
    const client = address.startsWith('https:') ? https : http;
    const request = client.get(address + report, { agent: false }, response => {
      if (response.statusCode !== 200) {
        response.resume();
        failed(`the response status was ${response.statusCode}`);
        return;
      }
      const chunks = [];
      response.on('data', chunk => chunks.push(chunk));
      response.on('error', error => failed(`the connection failed: ${error.message}`));
      response.on('end', () => {
        try {
          resolve({ phase, report: JSON.parse(Buffer.concat(chunks).toString('utf8')) });
        } catch {
          failed('the body is not JSON');
        }
      });
    });
    // The limit covers the whole read, so a body that never ends is as much a failure as no answer.
    const timer = setTimeout(() => {
      request.destroy();
      failed('no response within 10 seconds');
    }, READ_LIMIT_MS);
    request.on('error', error => failed(`the connection failed: ${error.message}`));
    request.on('close', () => clearTimeout(timer));
  });
}

// Runs inside the runner, as the first global setup and the last global teardown.
async function record(phase) {
  const plan = globalThis[PLAN];
  const reads = await Promise.all(plan.components.map(async component => ({
    name: component.name,
    read: await read(component.address, component.report, phase),
  })));
  fs.writeFileSync(path.join(plan.folder, `${phase}.json`), JSON.stringify(reads));
}

module.exports = { PLAN, addressOf, answers, record };
