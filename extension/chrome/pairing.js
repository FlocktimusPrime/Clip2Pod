// Pairing handshake with the app. Classic script shared by both builds
// (identical file): importScripts() in Chrome, background.scripts in Firefox.
//
// The pairing code never leaves the browser. When Clip2Pod isn't running, any
// local program can listen on its port, so the app has to prove it knows the
// code (an HMAC of our random nonce) before we send it anything; we then sign
// its single-use challenge the same way. Mirrors src-tauri/src/capture.rs.

const SERVER_LABEL = "clip2pod-server:";
const CLIENT_LABEL = "clip2pod-client:";

function toHex(buffer) {
  return [...new Uint8Array(buffer)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

// Lowercase hex HMAC-SHA256 of `label + data`, keyed by the pairing code.
async function pairingMac(code, label, data) {
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(code),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  return toHex(await crypto.subtle.sign("HMAC", key, enc.encode(label + data)));
}

function randomNonce() {
  return toHex(crypto.getRandomValues(new Uint8Array(16)).buffer);
}

// Constant-time-ish compare for equal-length hex strings.
function sameHex(a, b) {
  if (typeof a !== "string" || a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return diff === 0;
}

// Ask the app at `endpoint` (".../hello") to prove it knows `code`. Resolves to
// the X-Clip2Pod-Auth header value for one capture, or throws:
// "unreachable" (nothing listening), "impostor" (wrong proof: wrong code, or a
// program that isn't Clip2Pod).
async function pairingAuth(endpoint, code, fetchFn = fetch) {
  const nonce = randomNonce();
  let reply;
  try {
    const response = await fetchFn(endpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ nonce }),
    });
    reply = response.ok ? await response.json() : {};
  } catch {
    throw new Error("unreachable");
  }
  const expected = await pairingMac(code, SERVER_LABEL, nonce);
  if (!sameHex(reply.proof, expected) || typeof reply.challenge !== "string") {
    throw new Error("impostor");
  }
  return `${reply.challenge}.${await pairingMac(code, CLIENT_LABEL, reply.challenge)}`;
}
