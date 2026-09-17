let input = '';
process.stdin.on('data', (c) => (input += c));
process.stdin.on('end', () => {
  let payload;
  try {
    payload = JSON.parse(input || '{}');
  } catch {
    payload = {};
  }
  const file = (payload.tool_input?.file_path || '').replace(/\\/g, '/');
  if (!file.includes('src-tauri/crates/ytdlfeed-core/')) process.exit(0);

  console.log(
    JSON.stringify({
      hookSpecificOutput: {
        hookEventName: 'PreToolUse',
        permissionDecision: 'ask',
        permissionDecisionReason:
          'ytdlfeed-core is vendored via git subtree from the archived yt-dlFeed repo (GPL-3.0-only) — confirm this edit is intentional and not a casual change that would diverge from upstream.',
      },
    })
  );
});
