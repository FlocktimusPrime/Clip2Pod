const DEFAULT_COMMENT_CAP = 25;
const input = document.getElementById("cap");

function clampCap(value) {
  const n = Number.parseInt(value, 10);
  if (Number.isNaN(n)) return DEFAULT_COMMENT_CAP;
  return Math.min(500, Math.max(1, n));
}

async function load() {
  const { redditCommentCap } = await chrome.storage.sync.get({
    redditCommentCap: DEFAULT_COMMENT_CAP,
  });
  input.value = clampCap(redditCommentCap);
}

// Persist mid-typing so closing the dialog without blurring keeps the value.
// Only a valid number is stored, and the field is left alone while typing.
input.addEventListener("input", async () => {
  if (Number.isNaN(Number.parseInt(input.value, 10))) return;
  await chrome.storage.sync.set({ redditCommentCap: clampCap(input.value) });
});

input.addEventListener("change", async () => {
  const cap = clampCap(input.value);
  input.value = cap;
  await chrome.storage.sync.set({ redditCommentCap: cap });
});

load();
