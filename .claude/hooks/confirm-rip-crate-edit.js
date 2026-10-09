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
  if (!file.includes('src-tauri/crates/clip2pod-rip/')) process.exit(0);

  console.log(
    JSON.stringify({
      hookSpecificOutput: {
        hookEventName: 'PreToolUse',
        permissionDecision: 'ask',
        permissionDecisionReason:
          'clip2pod-rip is GPL-3.0-only while the rest of Clip2Pod is AGPL-3.0 — confirm this edit is intentional and keeps the licence boundary intact.',
      },
    })
  );
});
