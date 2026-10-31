import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const chromeSrc = fs.readFileSync(path.join(here, "../chrome/reddit.js"), "utf8");
const firefoxSrc = fs.readFileSync(path.join(here, "../firefox/reddit.js"), "utf8");

// Load reddit.js as a classic script in a bare context, as the browser does.
function load(tabUrl = "https://www.reddit.com/r/news/comments/abc123/slug/", response = {}) {
  const requested = [];
  const fetchStub = async (url) => {
    requested.push(url);
    return {
      ok: response.ok ?? true,
      status: response.status ?? 200,
      text: async () => response.body ?? "[]",
    };
  };
  const ctx = vm.createContext({
    URL,
    JSON,
    location: new URL(tabUrl),
    fetch: fetchStub,
  });
  vm.runInContext(chromeSrc, ctx);
  return { ctx, requested };
}

const POST = {
  title: "Big news",
  subreddit: "news",
  author: "op_user",
  selftext: "",
  is_self: true,
  domain: "self.news",
};
const COMMENT = {
  author: "alice",
  body: "this is a real comment here",
  score: 10,
  stickied: false,
  distinguished: null,
};

function listing(post, comments) {
  return JSON.stringify([
    { data: { children: [{ kind: "t3", data: { ...POST, ...post } }] } },
    {
      data: {
        children: comments.map((c) =>
          c.kind ? c : { kind: "t1", data: { ...COMMENT, ...c } },
        ),
      },
    },
  ]);
}

// Objects built inside the vm context have a foreign prototype, so results
// are JSON-cloned before comparing.
const clone = (v) => JSON.parse(JSON.stringify(v));

async function run(post, comments, { cap = 25, tabUrl } = {}) {
  const { ctx } = load(tabUrl, { body: listing(post, comments) });
  return clone(await ctx.extractRedditThread(cap));
}

// The stripped body of a single comment, without its header line.
async function commentText(body, extra = {}) {
  const r = await run({}, [{ body, ...extra }]);
  assert.ok(r.prepared, JSON.stringify(r));
  const t = r.prepared.text;
  const marker = "Top comments.\n\n";
  const afterTop = t.slice(t.indexOf(marker) + marker.length);
  return afterTop.slice(afterTop.indexOf("\n") + 1);
}

async function header(extra) {
  const r = await run({}, [extra]);
  return r.prepared.text.split("\n").find((l) => l.startsWith("Comment 1,"));
}

test("chrome and firefox reddit.js are identical", () => {
  assert.equal(chromeSrc, firefoxSrc);
});

test("chrome and firefox options page files are identical", () => {
  for (const f of ["options.html", "options.js"]) {
    assert.equal(
      fs.readFileSync(path.join(here, "../chrome", f), "utf8"),
      fs.readFileSync(path.join(here, "../firefox", f), "utf8"),
      f,
    );
  }
});

test("isRedditThread", () => {
  const { ctx } = load();
  const f = (u) => ctx.isRedditThread(u);
  for (const u of [
    "https://www.reddit.com/r/news/comments/abc123/slug/",
    "https://old.reddit.com/comments/abc123",
    "https://np.reddit.com/r/x/comments/abc123/slug/def456/",
    "https://www.reddit.com/r/x/comments/abc123/?utm_source=share#c",
    "https://www.reddit.com/user/bob/comments/abc123/slug/",
    "https://www.reddit.com/u/bob/comments/abc123/",
  ]) {
    assert.equal(f(u), true, u);
  }
  for (const u of [
    "https://www.reddit.com/r/news/",
    "https://www.reddit.com/user/x/comments/",
    "https://www.reddit.com/r/sub/comments/",
    "https://notreddit.com/comments/abc123",
    "https://reddit.com.evil.com/comments/abc123",
  ]) {
    assert.equal(f(u), false, u);
  }
});

test("fetch URL is built from the tab origin and thread id", async () => {
  const tab = "https://old.reddit.com/r/x/comments/abc123/slug/def456/";
  const { ctx, requested } = load(tab, { body: listing({}, []) });
  const r = clone(await ctx.extractRedditThread(25));
  assert.deepEqual(requested, [
    "https://old.reddit.com/comments/abc123.json?sort=top&depth=1&limit=500&raw_json=1",
  ]);
  assert.equal(r.url, tab);
});

test("user-profile thread paths fetch by id", async () => {
  for (const tab of [
    "https://www.reddit.com/user/bob/comments/abc123/slug/",
    "https://www.reddit.com/u/bob/comments/abc123/",
  ]) {
    const { ctx, requested } = load(tab, { body: listing({}, []) });
    const r = clone(await ctx.extractRedditThread(25));
    assert.deepEqual(requested, [
      "https://www.reddit.com/comments/abc123.json?sort=top&depth=1&limit=500&raw_json=1",
    ]);
    assert.equal(r.url, tab);
  }
});

test("prepared title and author", async () => {
  const r = await run({}, []);
  assert.equal(r.prepared.title, "Big news from r/news");
  assert.equal(r.prepared.author, "u/op_user");
  const d = await run({ author: "[deleted]" }, []);
  assert.equal(d.prepared.author, "");
});

test("self post layout", async () => {
  const r = await run({ selftext: "Hello **world** there." }, [
    {},
    { author: "bob", score: 1, body: "second comment with enough words" },
  ]);
  assert.equal(
    r.prepared.text,
    "Hello world there.\n\nTop comments.\n\n" +
      "Comment 1, from u/alice, 10 upvotes.\nthis is a real comment here\n\n" +
      "Comment 2, from u/bob, 1 upvote.\nsecond comment with enough words",
  );
});

test("link post uses the linked-article line", async () => {
  const r = await run({ is_self: false, domain: "nytimes.com" }, [{}]);
  assert.ok(
    r.prepared.text.startsWith("Linked article on nytimes.com.\n\nTop comments."),
    r.prepared.text,
  );
});

test("Reddit-hosted media posts name the media, not a domain", async () => {
  for (const [post, line] of [
    [{ is_self: false, domain: "i.redd.it" }, "Image post."],
    [{ is_self: false, domain: "v.redd.it" }, "Video post."],
    [{ is_self: false, domain: "reddit.com", is_gallery: true }, "Gallery post."],
  ]) {
    const r = await run(post, [{}]);
    assert.ok(r.prepared.text.startsWith(`${line}\n\nTop comments.`), r.prepared.text);
  }
});

test("empty self body leaves no leading gap", async () => {
  for (const selftext of ["", "![img](https://preview.redd.it/x.png)", "[removed]", "[deleted]"]) {
    const r = await run({ selftext }, [{}]);
    assert.ok(r.prepared.text.startsWith("Top comments.\n\n"), JSON.stringify(selftext));
  }
});

test("filters drop junk comments", async () => {
  const junk = [
    { body: "[deleted]" },
    { body: "[removed]" },
    { author: "[deleted]" },
    { author: "AutoModerator" },
    { stickied: true },
    { distinguished: "moderator" },
    { body: "https://imgur.com/x" },
    { body: "![gif](giphy|abc)" },
    { body: "lol same" },
    { kind: "more", data: { children: ["x"] } },
  ];
  for (const j of junk) {
    const r = await run({}, [j, { author: "keeper" }]);
    const headers = r.prepared.text.match(/^Comment \d+,.*$/gm);
    assert.deepEqual(headers, ["Comment 1, from u/keeper, 10 upvotes."], JSON.stringify(j));
  }
});

test("cap applies after filtering", async () => {
  const comments = [
    { body: "[deleted]" },
    { author: "AutoModerator" },
    { stickied: true },
    ...Array.from({ length: 30 }, (_, i) => ({ body: `good comment number ${i + 1} here` })),
  ];
  const r = await run({}, comments, { cap: 25 });
  const headers = r.prepared.text.match(/^Comment \d+,/gm);
  assert.equal(headers.length, 25);
  assert.equal(headers[24], "Comment 25,");
  const bodies = r.prepared.text.match(/^good comment number \d+ here$/gm);
  assert.deepEqual(
    bodies,
    Array.from({ length: 25 }, (_, i) => `good comment number ${i + 1} here`),
  );
});

test("no surviving comments", async () => {
  const r = await run({}, [{ body: "[deleted]" }, { author: "AutoModerator" }]);
  assert.equal(r.prepared.text, "No comments.");
  assert.ok(!r.prepared.text.includes("Top comments."));
  const withBody = await run({ selftext: "A post body." }, []);
  assert.equal(withBody.prepared.text, "A post body.\n\nNo comments.");
});

test("score phrases", async () => {
  const cases = [
    [0, "0 upvotes"],
    [1, "1 upvote"],
    [840, "840 upvotes"],
    [999, "999 upvotes"],
    [1000, "1 thousand upvotes"],
    [1234, "1.2 thousand upvotes"],
    [15000, "15 thousand upvotes"],
    [999950, "1 million upvotes"],
    [1100000, "1.1 million upvotes"],
    [-4, "-4 upvotes"],
  ];
  for (const [score, phrase] of cases) {
    assert.equal(await header({ score }), `Comment 1, from u/alice, ${phrase}.`, String(score));
  }
  assert.equal(await header({ score_hidden: true }), "Comment 1, from u/alice.");
});

test("markdown stripping", async () => {
  const cases = [
    ["see [the docs](https://x.com/d) for more info", "see the docs for more info"],
    ["read this https://x.com/a?b=c right now please", "read this right now please"],
    ["> quoted words here\n\nmy reply is longer", "quoted words here\n\nmy reply is longer"],
    [
      "# Heading\n*bold* _it_ ~~gone~~ ^super and `code` here",
      "Heading\nbold it gone super and code here",
    ],
    ["ping u/some_user about my_var_name today ok", "ping u/some_user about my_var_name today ok"],
    ["para one words here\n\n\n\npara two words here", "para one words here\n\npara two words here"],
    ["see [wiki](https://x.org/a_(b)) for more details", "see wiki for more details"],
    ["careful >!the butler did it!< ok then", "careful the butler did it ok then"],
    [">!the butler did it!< is the twist", "the butler did it is the twist"],
  ];
  for (const [input, expected] of cases) {
    assert.equal(await commentText(input), expected, input);
  }

  const fenced = await commentText("```\nlet x = 1;\n``` plus four more words");
  assert.ok(fenced.includes("let x = 1;"), fenced);
  assert.ok(!fenced.includes("`"), fenced);

  const table = await commentText("| a | b |\n|---|---|\n| one | two | three four five");
  assert.ok(!table.includes("|"), table);
  assert.ok(!table.includes("---"), table);
  for (const w of ["a", "b", "one", "two", "three", "four", "five"]) {
    assert.ok(table.split(/\s+/).includes(w), `${w} in ${table}`);
  }
});

test("errors", async () => {
  const forbidden = load(undefined, { ok: false, status: 403 });
  assert.deepEqual(clone(await forbidden.ctx.extractRedditThread(25)), {
    error: "Reddit returned 403",
  });
  const html = load(undefined, { body: "<html>" });
  assert.deepEqual(clone(await html.ctx.extractRedditThread(25)), {
    error: "Reddit response was not JSON",
  });
  const notThread = load("https://www.reddit.com/r/news/");
  assert.deepEqual(clone(await notThread.ctx.extractRedditThread(25)), {
    error: "not a Reddit thread",
  });
});

test("extractRedditThread is self-contained when serialized", async () => {
  const { ctx } = load();
  const src = ctx.extractRedditThread.toString();
  const requested = [];
  const fresh = vm.createContext({
    location: new URL("https://www.reddit.com/r/news/comments/abc123/slug/"),
    fetch: async (url) => {
      requested.push(url);
      return { ok: true, status: 200, text: async () => listing({}, [{}]) };
    },
  });
  const fn = vm.runInContext("(" + src + ")", fresh);
  const r = clone(await fn(25));
  assert.deepEqual(requested, [
    "https://www.reddit.com/comments/abc123.json?sort=top&depth=1&limit=500&raw_json=1",
  ]);
  assert.equal(r.url, "https://www.reddit.com/r/news/comments/abc123/slug/");
  assert.ok(r.prepared?.text.includes("Comment 1, from u/alice"), JSON.stringify(r));
});
