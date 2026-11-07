#!/usr/bin/env node
/**
 * Verify an .flse manifest signature against the committed public key.
 *
 * Usage:
 *   node scripts/verify-manifest.mjs <manifest.json> <manifest.sig> [public-key]
 */

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

const [manifestPath, sigPath, pubArg] = process.argv.slice(2);
if (!manifestPath || !sigPath) {
  console.error('usage: node scripts/verify-manifest.mjs <manifest.json> <manifest.sig> [public-key]');
  process.exit(2);
}

const pubPath =
  pubArg || path.resolve(process.cwd(), 'crates/false-ui/keys/update-signing.pub');
const raw = Buffer.from(fs.readFileSync(pubPath, 'utf8').trim(), 'base64');
if (raw.length !== 32) throw new Error(`public key must be 32 bytes, got ${raw.length}`);

// Wrap the raw key in a minimal SPKI DER so Node can import it.
const prefix = Buffer.from('302a300506032b6570032100', 'hex');
const publicKey = crypto.createPublicKey({
  key: Buffer.concat([prefix, raw]),
  format: 'der',
  type: 'spki',
});

const data = fs.readFileSync(manifestPath);
const signature = Buffer.from(fs.readFileSync(sigPath, 'utf8').trim(), 'base64');

if (!crypto.verify(null, data, publicKey, signature)) {
  console.error(`INVALID signature for ${manifestPath}`);
  process.exit(1);
}

console.log(`Signature OK: ${path.basename(manifestPath)}`);
