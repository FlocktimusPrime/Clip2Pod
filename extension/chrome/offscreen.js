// Hidden page whose only job is to watch the browser's colour scheme: the
// service worker has no matchMedia, so it can't tell light from dark itself.
// Reports on load and on every change; background.js picks the toolbar icon.

const dark = matchMedia("(prefers-color-scheme: dark)");
const report = () => chrome.runtime.sendMessage({ type: "c2p-scheme", dark: dark.matches });

dark.addEventListener("change", report);
report();
