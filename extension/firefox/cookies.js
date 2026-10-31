// Per-site cookie access for rips. Classic script shared by both builds
// (identical file): importScripts() in Chrome, background.scripts in Firefox.

// Public suffixes with two labels, so "www.bbc.co.uk" -> "bbc.co.uk".
const TWO_PART_SUFFIXES = new Set([
  "co.uk", "org.uk", "ac.uk", "gov.uk", "com.au", "net.au", "org.au",
  "co.nz", "co.jp", "co.in", "com.br", "co.za",
]);

// Registrable domain of a host, or null for IPs, localhost and single labels.
function siteBase(host) {
  host = String(host).toLowerCase();
  if (host.startsWith("[") || /^\d+\.\d+\.\d+\.\d+$/.test(host)) return null;
  const labels = host.split(".");
  if (labels.length < 2 || host === "localhost") return null;
  const take = TWO_PART_SUFFIXES.has(labels.slice(-2).join(".")) ? 3 : 2;
  return labels.slice(-take).join(".");
}

function sitePattern(url) {
  try {
    const base = siteBase(new URL(url).hostname);
    return base && "*://*." + base + "/*";
  } catch {
    return null;
  }
}

// Not async: permissions.request must run inside the click's user gesture,
// before any await.
function requestCookieAccess(url) {
  const pattern = sitePattern(url);
  if (!pattern) return Promise.resolve(false);
  try {
    return Promise.resolve(chrome.permissions.request({ origins: [pattern] })).then(
      (granted) => granted === true,
      () => false,
    );
  } catch {
    return Promise.resolve(false);
  }
}

function hasCookieAccess(url) {
  const pattern = sitePattern(url);
  if (!pattern) return Promise.resolve(false);
  try {
    return Promise.resolve(chrome.permissions.contains({ origins: [pattern] })).then(
      (has) => has === true,
      () => false,
    );
  } catch {
    return Promise.resolve(false);
  }
}

// The tab's cookies for its URL, or undefined (no access, none found, or any
// error). `access` is the promise from requestCookieAccess/hasCookieAccess.
async function collectCookies(tab, access) {
  try {
    if (!(await access)) return undefined;
    const storeId =
      tab.cookieStoreId ??
      (await chrome.cookies.getAllCookieStores()).find((s) => s.tabIds.includes(tab.id))?.id;
    // Unknown store: never fall back to the default store (wrong profile/container).
    if (storeId === undefined) return undefined;
    const cookies = await chrome.cookies.getAll({ url: tab.url, storeId });
    if (!cookies.length) return undefined;
    return cookies.map((c) => ({
      domain: c.domain,
      hostOnly: c.hostOnly,
      path: c.path,
      secure: c.secure,
      expirationDate: c.expirationDate,
      name: c.name,
      value: c.value,
    }));
  } catch (e) {
    console.warn("Clip2Pod: could not read cookies:", e);
    return undefined;
  }
}
