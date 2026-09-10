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
