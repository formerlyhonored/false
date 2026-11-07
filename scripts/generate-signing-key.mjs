#!/usr/bin/env node
/**
 * Generate the Ed25519 update-signing keypair.
 *
 * Private key: %LOCALAPPDATA%\False\signing\update-signing.key.pem (never commit)
 * Public key:  crates/false-ui/keys/update-signing.pub (raw 32 bytes, base64)
 *
 * Usage: node scripts/generate-signing-key.mjs [--force]
 */

import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const force = process.argv.includes('--force');
const keyDir = process.env.LOCALAPPDATA
  ? path.join(process.env.LOCALAPPDATA, 'False', 'signing')
  : path.join(os.homedir(), '.false', 'signing');
const keyPath = path.join(keyDir, 'update-signing.key.pem');
const pubPath = path.resolve(process.cwd(), 'crates/false-ui/keys/update-signing.pub');

if (fs.existsSync(keyPath) && !force) {
  console.error(`Refusing to overwrite existing key: ${keyPath}`);
  console.error('Pass --force only if you intend to rotate the release key (old packages will stop verifying).');
  process.exit(1);
}

fs.mkdirSync(keyDir, { recursive: true });
const { publicKey, privateKey } = crypto.generateKeyPairSync('ed25519');

const pkcs8 = privateKey.export({ type: 'pkcs8', format: 'pem' });
fs.writeFileSync(keyPath, pkcs8, { mode: 0o600 });

// Node's SPKI DER for Ed25519 ends with the raw 32-byte public key.
const spki = publicKey.export({ type: 'spki', format: 'der' });
const raw = spki.subarray(spki.length - 32);
if (raw.length !== 32) throw new Error(`unexpected public key length: ${raw.length}`);

fs.mkdirSync(path.dirname(pubPath), { recursive: true });
fs.writeFileSync(pubPath, raw.toString('base64') + '\n');

const fingerprint = crypto
  .createHash('sha256')
  .update(raw)
  .digest('hex')
  .match(/.{2}/g)
  .join(':');

console.log('Generated Ed25519 update-signing keypair');
console.log(`  private: ${keyPath}`);
console.log(`  public:  ${pubPath}`);
console.log(`  sha256:  ${fingerprint}`);
console.log('\nCommit the public key only. Back up the private key somewhere safe.');
