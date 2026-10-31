// Reddit thread capture. Classic script (no modules): Chrome loads it with
// importScripts, Firefox via manifest "background.scripts". Keep
// chrome/reddit.js and firefox/reddit.js byte-identical.

// True for a Reddit thread tab: reddit.com (or a subdomain) with a
// /comments/<id> path, optionally under /r/<sub> or a /u(ser)/<name> profile.
// Runs in the background.
function isRedditThread(url) {
  let u;
  try {
    u = new URL(url);
  } catch {
    return false;
  }
  const host = u.hostname.toLowerCase();
  if (host !== "reddit.com" && !host.endsWith(".reddit.com")) return false;
  return /^(?:\/(?:r|u|user)\/[^/]+)?\/comments\/[a-z0-9]+(?:\/|$)/i.test(
    u.pathname,
  );
}

// Injected into the thread tab with chrome.scripting.executeScript, which
// serializes this function's source: it must stay self-contained (every
// helper nested, no references to other top-level names).
async function extractRedditThread(cap) {
  const MAX_COMMENTS = 500;
  const DEFAULT_COMMENTS = 25;
  const MIN_WORDS = 5;

  const stripMarkdown = (md) => {
    const paren = String.raw`\((?:[^()]|\([^()]*\))*\)`;
    let s = String(md ?? "").replace(/\r\n?/g, "\n");
    s = s
      // 1. image/GIF embeds
      .replace(new RegExp(String.raw`!\[[^\]]*\]${paren}`, "g"), "")
      .replace(/giphy\|\S+/g, "")
      .replace(/\S*preview\.redd\.it\S*/g, "")
      // 2. [text](url) -> text
      .replace(new RegExp(String.raw`\[([^\]]*)\]${paren}`, "g"), "$1")
      // 3. bare URLs
      .replace(/https?:\/\/\S+/g, "")
      // 4. code: drop fence-only lines, then any remaining backticks
      .replace(/^[ \t]*```[\w+-]*[ \t]*$/gm, "")
      .replace(/`/g, "");
    // 5. tables: drop separator rows, pipes become spaces
    s = s
      .split("\n")
      .filter((l) => !(l.includes("-") && /^\s*\|?[\s:|-]+\|?\s*$/.test(l)))
      .join("\n")
      .replace(/\|/g, " ");
    s = s
      // 6. spoilers (>!text!< keeps the words), quote markers, headings
      .replace(/>!([\s\S]*?)!</g, "$1")
      .replace(/^[ \t]*(?:>[ \t]*)+/gm, "")
      .replace(/^[ \t]*#{1,6}[ \t]+/gm, "")
      // 7. emphasis, only where it wraps text at word boundaries
      .replace(/~~(.+?)~~/g, "$1")
      .replace(/(^|\W)([*_]{1,2})(\S.*?\S|\S)\2(?=\W|$)/gm, "$1$3")
      .replace(/\^/g, "");
    // 8. whitespace
    return s
      .split("\n")
      .map((l) => l.replace(/[ \t]+/g, " ").trim())
      .join("\n")
      .replace(/\n{3,}/g, "\n\n")
      .trim();
  };

  const scorePhrase = (n) => {
    if (n < 1000) return `${n} upvote${n === 1 ? "" : "s"}`;
    const thousands = Math.round(n / 100) / 10;
    if (thousands < 1000) return `${thousands} thousand upvotes`;
    return `${Math.round(n / 1e5) / 10} million upvotes`;
  };

  const countWords = (s) => s.split(/\s+/).filter(Boolean).length;

  try {
    const id = /^(?:\/(?:r|u|user)\/[^/]+)?\/comments\/([a-z0-9]+)(?:\/|$)/i.exec(
      location.pathname,
    )?.[1];
    if (!id) return { error: "not a Reddit thread" };

    const limit = Math.min(
      MAX_COMMENTS,
      Math.max(1, Math.floor(Number(cap)) || DEFAULT_COMMENTS),
    );

    const response = await fetch(
      `${location.origin}/comments/${id}.json?sort=top&depth=1&limit=${MAX_COMMENTS}&raw_json=1`,
    );
    if (!response.ok) return { error: `Reddit returned ${response.status}` };

    let listing;
    try {
      listing = JSON.parse(await response.text());
    } catch {
      return { error: "Reddit response was not JSON" };
    }

    const post = listing?.[0]?.data?.children?.[0]?.data;
    if (!post) return { error: "Reddit response was not a thread" };
    const children = listing[1]?.data?.children ?? [];

    const entries = [];
    for (const child of children) {
      if (entries.length >= limit) break;
      if (child?.kind !== "t1") continue;
      const c = child.data ?? {};
      const raw = String(c.body ?? "").trim();
      if (raw === "[deleted]" || raw === "[removed]") continue;
      if (c.author === "[deleted]" || c.author === "AutoModerator") continue;
      if (c.stickied || c.distinguished === "moderator") continue;
      const body = stripMarkdown(raw);
      if (!body || countWords(body) < MIN_WORDS) continue;

      let header = `Comment ${entries.length + 1}, from u/${c.author}`;
      if (!c.score_hidden && Number.isFinite(c.score)) {
        header += `, ${scorePhrase(c.score)}`;
      }
      entries.push(`${header}.\n${body}`);
    }

    const bodyPart = post.is_self
      ? post.selftext?.trim() === "[removed]" ||
        post.selftext?.trim() === "[deleted]"
        ? ""
        : stripMarkdown(post.selftext)
      : post.is_gallery
        ? "Gallery post."
        : post.domain === "i.redd.it"
          ? "Image post."
          : post.domain === "v.redd.it"
            ? "Video post."
            : `Linked article on ${post.domain}.`;
    const commentsPart = entries.length
      ? `Top comments.\n\n${entries.join("\n\n")}`
      : "No comments.";

    return {
      url: location.href,
      prepared: {
        title: `${post.title} from r/${post.subreddit}`,
        author:
          !post.author || post.author === "[deleted]" ? "" : `u/${post.author}`,
        text: [bodyPart, commentsPart].filter(Boolean).join("\n\n"),
      },
    };
  } catch (e) {
    return { error: `Reddit capture failed: ${e?.message ?? e}` };
  }
}
