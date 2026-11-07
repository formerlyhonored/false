#!/usr/bin/env node
/**
 * Sign an .flse manifest with the release Ed25519 key.
 *
 * Usage:
 *   node scripts/sign-manifest.mjs <manifest.json> <manifest.sig>
 *
 * Private key resolution:
 *   1. FALSE_SIGNING_KEY environment variable
 *   2. %LOCALAPPDATA%\False\signing\update-signing.key.pem
 */

import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const [manifestPath, sigPath] = process.argv.slice(2);
if (!manifestPath || !sigPath) {
  console.error('usage: node scripts/sign-manifest.mjs <manifest.json> <manifest.sig>');
  process.exit(2);
}

const keyPath =
  process.env.FALSE_SIGNING_KEY ||
  (process.env.LOCALAPPDATA
    ? path.join(process.env.LOCALAPPDATA, 'False', 'signing', 'update-signing.key.pem')
    : path.join(os.homedir(), '.false', 'signing', 'update-signing.key.pem'));

if (!fs.existsSync(keyPath)) {
  console.error(`signing key not found: ${keyPath}`);
  console.error('Run `node scripts/generate-signing-key.mjs` first.');
  process.exit(1);
}

const privateKey = crypto.createPrivateKey({
  key: fs.readFileSync(keyPath),
  format: 'pem',
  type: 'pkcs8',
});

// Ed25519 uses a null digest: the algorithm hashes internally.
const data = fs.readFileSync(manifestPath);
const signature = crypto.sign(null, data, privateKey);
fs.writeFileSync(sigPath, signature.toString('base64') + '\n');

console.log(`Signed ${path.basename(manifestPath)} -> ${path.basename(sigPath)}`);
