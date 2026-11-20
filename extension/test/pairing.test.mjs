import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { createHmac } from "node:crypto";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const src = fs.readFileSync(path.join(here, "../chrome/pairing.js"), "utf8");

// Load pairing.js as a classic script, as the browser does.
function load() {
  const ctx = vm.createContext({ crypto: globalThis.crypto, TextEncoder, Uint8Array, Error, JSON });
  vm.runInContext(src, ctx);
  return ctx;
}

const CODE = "0123456789abcdef0123456789abcdef";
const hmac = (key, msg) => createHmac("sha256", key).update(msg).digest("hex");

test("both extension builds ship the same pairing.js", () => {
  assert.equal(fs.readFileSync(path.join(here, "../firefox/pairing.js"), "utf8"), src);
});

test("pairingMac is HMAC-SHA256 and matches the app's vector", async () => {
  const p = load();
  // RFC 4231 test case 2.
  assert.equal(
    await p.pairingMac("Jefe", "", "what do ya want for nothing?"),
    "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843",
  );
  // Same vector as capture.rs mac_is_standard_hmac_sha256_and_matches_the_extension.
  assert.equal(
    await p.pairingMac(CODE, "clip2pod-client:", "fedcba9876543210fedcba9876543210"),
    "0779ba6aa5d75bc7771501b5a2327ec77726a163b08002249db8a42f9bcc7749",
  );
});

// A fake /hello that answers as the real app would when it knows `code`.
function fakeApp(code, challenge = "c".repeat(32)) {
  const calls = [];
  const fetchFn = async (url, init) => {
    calls.push({ url, init });
    const { nonce } = JSON.parse(init.body);
    return {
      ok: true,
      json: async () => ({ proof: hmac(code, "clip2pod-server:" + nonce), challenge }),
    };
  };
  return { fetchFn, calls };
}

test("a real app gets a signed challenge, and the code is never sent", async () => {
  const p = load();
  const app = fakeApp(CODE);
  const auth = await p.pairingAuth("http://x/hello", CODE, app.fetchFn);
  const challenge = "c".repeat(32);
  assert.equal(auth, `${challenge}.${hmac(CODE, "clip2pod-client:" + challenge)}`);
  assert.equal(app.calls.length, 1);
  assert.ok(!JSON.stringify(app.calls).includes(CODE));
  assert.match(JSON.parse(app.calls[0].init.body).nonce, /^[0-9a-f]{32}$/);
});

test("an app that doesn't know the code is treated as an impostor", async () => {
  const p = load();
  const imposter = fakeApp("ffffffffffffffffffffffffffffffff");
  await assert.rejects(p.pairingAuth("http://x/hello", CODE, imposter.fetchFn), /impostor/);
  const garbage = async () => ({ ok: true, json: async () => ({ challenge: "x" }) });
  await assert.rejects(p.pairingAuth("http://x/hello", CODE, garbage), /impostor/);
  const refused = async () => ({ ok: false, json: async () => ({}) });
  await assert.rejects(p.pairingAuth("http://x/hello", CODE, refused), /impostor/);
});

test("nothing listening reads as unreachable", async () => {
  const p = load();
  const down = async () => {
    throw new TypeError("Failed to fetch");
  };
  await assert.rejects(p.pairingAuth("http://x/hello", CODE, down), /unreachable/);
});
