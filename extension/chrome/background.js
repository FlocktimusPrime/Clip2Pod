// Toolbar click: grab the rendered page from the active tab (the user's own
// logged-in session, so paywalled content is included) and post it to the
// Clip2Pod capture listener.

const CAPTURE_ENDPOINT = "http://127.0.0.1:4737/capture";

function badge(tabId, text, color) {
  chrome.action.setBadgeText({ tabId, text });
  chrome.action.setBadgeBackgroundColor({ tabId, color });
  setTimeout(() => chrome.action.setBadgeText({ tabId, text: "" }), 4000);
}

chrome.action.onClicked.addListener(async (tab) => {
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
      body: JSON.stringify(result),
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
});
