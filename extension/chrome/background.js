// Grab the rendered page from the active tab (the user's own logged-in
// session, so paywalled content is included) and post it to the Clip2Pod
// capture listener. A plain toolbar click lets the app decide the mode
// (article vs video); the right-click menu forces one.

const CAPTURE_ENDPOINT = "http://127.0.0.1:4737/capture";

function badge(tabId, text, color) {
  chrome.action.setBadgeText({ tabId, text });
  chrome.action.setBadgeBackgroundColor({ tabId, color });
  setTimeout(() => chrome.action.setBadgeText({ tabId, text: "" }), 4000);
}

// overrideHint: "article" | "video" | undefined (undefined = let the app route)
async function send(tab, overrideHint) {
  try {
    const [{ result }] = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: () => ({
        url: location.href,
        html: document.documentElement.outerHTML,
      }),
    });

    const response = await fetch(CAPTURE_ENDPOINT, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ ...result, override_hint: overrideHint }),
    });

    if (response.ok) {
      badge(tab.id, "✓", "#7bae6c");
    } else {
      console.error("Clip2Pod rejected the page:", await response.text());
      badge(tab.id, "!", "#e4483c");
    }
  } catch (e) {
    // Most common cause: Clip2Pod is not running (connection refused).
    console.error("Could not reach Clip2Pod:", e);
    badge(tab.id, "!", "#e4483c");
  }
}

chrome.action.onClicked.addListener((tab) => send(tab, undefined));

// Chrome has no manifest equivalent of Firefox's theme_icons, so swap the
// toolbar icon at runtime: the default (dark lavender) reads on light
// toolbars, the -light variant on dark ones. offscreen.js reports the scheme.
// Follows Chrome's light/dark mode; a custom dark theme isn't detectable.
const ICONS = {
  light: { 16: "icons/icon16.png", 32: "icons/icon32.png" },
  dark: { 16: "icons/icon16-light.png", 32: "icons/icon32-light.png" },
};

chrome.runtime.onMessage.addListener((msg) => {
  if (msg?.type === "c2p-scheme") {
    chrome.action.setIcon({ path: msg.dark ? ICONS.dark : ICONS.light });
  }
});

// One offscreen document at most per extension; it stays open to catch
// scheme changes. Runs on every service-worker start, so a browser restart
// or extension reload recreates it.
async function watchColorScheme() {
  const url = chrome.runtime.getURL("offscreen.html");
  const open = await chrome.runtime.getContexts({
    contextTypes: ["OFFSCREEN_DOCUMENT"],
    documentUrls: [url],
  });
  if (open.length > 0) return;
  try {
    await chrome.offscreen.createDocument({
      url: "offscreen.html",
      reasons: ["MATCH_MEDIA"],
      justification: "Pick a toolbar icon that reads on the browser's light or dark theme.",
    });
  } catch (e) {
    // Lost a race with a concurrent start; the other call created it.
    if (!String(e).includes("single offscreen")) console.error("Clip2Pod icon watcher:", e);
  }
}

watchColorScheme();

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.removeAll(() => {
    chrome.contextMenus.create({
      id: "c2p-article",
      title: "Narrate this page with Clip2Pod",
      contexts: ["page", "selection"],
    });
    chrome.contextMenus.create({
      id: "c2p-video",
      title: "Rip this page's audio with Clip2Pod",
      contexts: ["page", "selection"],
    });
  });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (!tab) return;
  send(tab, info.menuItemId === "c2p-video" ? "video" : "article");
});
