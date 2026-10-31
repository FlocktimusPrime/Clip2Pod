import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const src = fs.readFileSync(path.join(here, "../chrome/cookies.js"), "utf8");

// Load cookies.js as a classic script with a stub `chrome`, as the browser does.
function load(chrome = {}) {
  const warnings = [];
  const ctx = vm.createContext({
    URL,
    chrome,
    console: { warn: (...a) => warnings.push(a), error() {}, log() {} },
  });
  vm.runInContext(src, ctx);
  return { ctx, warnings };
}

const plain = (v) => JSON.parse(JSON.stringify(v));

test("siteBase finds the registrable domain", () => {
  const { ctx } = load();
  for (const h of ["www.reddit.com", "old.reddit.com", "reddit.com"]) {
    assert.equal(ctx.siteBase(h), "reddit.com");
  }
  assert.equal(ctx.siteBase("m.youtube.com"), "youtube.com");
  assert.equal(ctx.siteBase("www.bbc.co.uk"), "bbc.co.uk");
});

test("siteBase is null for IPs, localhost and single labels (RF5)", () => {
  const { ctx } = load();
  for (const h of ["127.0.0.1", "localhost", "[::1]", "intranet"]) {
    assert.equal(ctx.siteBase(h), null, h);
  }
});

test("sitePattern builds a wildcard origin pattern", () => {
  const { ctx } = load();
  assert.equal(ctx.sitePattern("https://old.reddit.com/r/x/comments/a/"), "*://*.reddit.com/*");
  assert.equal(ctx.sitePattern("http://localhost:3000/"), null);
  assert.equal(ctx.sitePattern("not a url"), null);
});

test("requestCookieAccess calls permissions.request synchronously", async () => {
  const calls = [];
  const { ctx } = load({
    permissions: { request: (arg) => (calls.push(arg), Promise.resolve(true)) },
  });
  const p = ctx.requestCookieAccess("https://www.reddit.com/r/x");
  assert.equal(calls.length, 1, "request must be called before any await");
  assert.deepEqual(plain(calls[0]), { origins: ["*://*.reddit.com/*"] });
  assert.equal(await p, true);
});

test("requestCookieAccess resolves false on rejection", async () => {
  const { ctx } = load({ permissions: { request: () => Promise.reject(new Error("no")) } });
  assert.equal(await ctx.requestCookieAccess("https://www.reddit.com/"), false);
});

test("requestCookieAccess skips hosts with no registrable domain (RF5)", async () => {
  const calls = [];
  const { ctx } = load({ permissions: { request: (a) => (calls.push(a), Promise.resolve(true)) } });
  assert.equal(await ctx.requestCookieAccess("http://localhost:3000/"), false);
  assert.equal(await ctx.requestCookieAccess(undefined), false);
  assert.equal(calls.length, 0);
});

test("hasCookieAccess uses permissions.contains", async () => {
  const calls = [];
  const { ctx } = load({
    permissions: { contains: (a) => (calls.push(a), Promise.resolve(true)) },
  });
  assert.equal(await ctx.hasCookieAccess("https://m.youtube.com/watch?v=1"), true);
  assert.deepEqual(plain(calls[0]), { origins: ["*://*.youtube.com/*"] });
  assert.equal(await ctx.hasCookieAccess("http://127.0.0.1/"), false);
  const bad = load({ permissions: { contains: () => Promise.reject(new Error("x")) } });
  assert.equal(await bad.ctx.hasCookieAccess("https://www.reddit.com/"), false);
});

const RAW = {
  domain: ".reddit.com",
  hostOnly: false,
  path: "/",
  secure: true,
  expirationDate: 1900000000.5,
  name: "sid",
  value: "abc",
  httpOnly: true,
  sameSite: "lax",
  storeId: "0",
};

test("collectCookies is undefined without access and never reads cookies (RF4)", async () => {
  let called = 0;
  const { ctx } = load({
    cookies: { getAll: async () => (called++, [RAW]), getAllCookieStores: async () => [] },
  });
  const tab = { id: 1, url: "https://www.reddit.com/" };
  assert.equal(await ctx.collectCookies(tab, Promise.resolve(false)), undefined);
  assert.equal(called, 0);
});

test("collectCookies uses the Firefox cookieStoreId", async () => {
  const seen = [];
  const { ctx } = load({ cookies: { getAll: async (q) => (seen.push(q), [RAW]) } });
  const tab = { id: 3, url: "https://www.reddit.com/", cookieStoreId: "firefox-container-1" };
  const out = await ctx.collectCookies(tab, Promise.resolve(true));
  assert.deepEqual(plain(seen[0]), { url: tab.url, storeId: "firefox-container-1" });
  assert.equal(out.length, 1);
});

test("collectCookies finds the Chrome store by tab id and maps exactly 7 keys", async () => {
  const seen = [];
  const { ctx } = load({
    cookies: {
      getAll: async (q) => (seen.push(q), [RAW]),
      getAllCookieStores: async () => [
        { id: "0", tabIds: [1] },
        { id: "1", tabIds: [7] },
      ],
    },
  });
  const out = await ctx.collectCookies({ id: 7, url: "https://www.reddit.com/" }, Promise.resolve(true));
  assert.equal(seen[0].storeId, "1");
  assert.deepEqual(plain(out), [
    {
      domain: ".reddit.com",
      hostOnly: false,
      path: "/",
      secure: true,
      expirationDate: 1900000000.5,
      name: "sid",
      value: "abc",
    },
  ]);
  assert.deepEqual(Object.keys(out[0]).sort(), [
    "domain", "expirationDate", "hostOnly", "name", "path", "secure", "value",
  ]);
});

test("collectCookies returns undefined when the tab's store is unknown", async () => {
  let called = 0;
  const { ctx } = load({
    cookies: {
      getAll: async () => (called++, [RAW]),
      getAllCookieStores: async () => [{ id: "0", tabIds: [1] }],
    },
  });
  const out = await ctx.collectCookies({ id: 7, url: "https://www.reddit.com/" }, Promise.resolve(true));
  assert.equal(out, undefined);
  assert.equal(called, 0);
});

test("collectCookies degrades to undefined on error or empty (RF4)", async () => {
  const tab = { id: 1, url: "https://www.reddit.com/", cookieStoreId: "0" };
  const boom = load({
    cookies: {
      getAll: async () => {
        throw new Error("boom");
      },
    },
  });
  assert.equal(await boom.ctx.collectCookies(tab, Promise.resolve(true)), undefined);
  assert.equal(boom.warnings.length, 1);
  const empty = load({ cookies: { getAll: async () => [] } });
  assert.equal(await empty.ctx.collectCookies(tab, Promise.resolve(true)), undefined);
});
