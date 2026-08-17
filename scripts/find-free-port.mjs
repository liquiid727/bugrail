#!/usr/bin/env node
/**
 * Find the first free loopback port starting at a preferred port.
 *
 * Usage:
 *   node scripts/find-free-port.mjs [preferredPort] [maxPort]
 *
 * Prints the first free port (>= preferredPort, <= maxPort) to stdout and
 * exits 0. Exits 1 with a message on stderr when no free port is found.
 *
 * Probes both IPv4 (127.0.0.1) and IPv6 (::) so the reported port is usable
 * for a dev server bound to the loopback interface on either stack.
 */
import net from "node:net";

const PREFERRED = Number(process.argv[2] ?? 3011);
const MAX = Number(process.argv[3] ?? 65_535);
const HOSTS = ["127.0.0.1", "::"];

function probe(port, host) {
  return new Promise((resolve) => {
    const server = net.createServer();
    const done = (result) => {
      server.removeAllListeners();
      resolve(result);
    };
    server.once("error", (error) => {
      done(error.code === "EAFNOSUPPORT" || error.code === "EADDRNOTAVAIL" ? "unsupported" : "occupied");
    });
    server.listen(port, host, () => server.close(() => done("available")));
  });
}

async function isFree(port) {
  for (const host of HOSTS) {
    const result = await probe(port, host);
    if (result === "occupied") return false;
    // "unsupported" on a host (e.g. no IPv6 stack) is fine — keep probing others.
  }
  return true;
}

if (!Number.isInteger(PREFERRED) || PREFERRED < 0 || !Number.isInteger(MAX) || MAX > 65_535 || PREFERRED > MAX) {
  process.stderr.write(`Usage: node scripts/find-free-port.mjs [preferredPort] [maxPort]\n`);
  process.exit(2);
}

for (let port = PREFERRED; port <= MAX; port += 1) {
  if (await isFree(port)) {
    process.stdout.write(String(port));
    process.exit(0);
  }
}
process.stderr.write(`No free loopback port found in ${PREFERRED}..${MAX}.\n`);
process.exit(1);
