const { execSync } = require('child_process');

let input = '';
process.stdin.on('data', (c) => (input += c));
process.stdin.on('end', () => {
  let payload;
  try {
    payload = JSON.parse(input || '{}');
  } catch {
    payload = {};
  }
  const file = payload.tool_input?.file_path || '';
  if (!/\.(svelte|ts)$/.test(file)) process.exit(0);

  try {
    execSync('npx svelte-check --tsconfig ./tsconfig.json', { stdio: 'pipe' });
  } catch (e) {
    const output = `${e.stdout || ''}${e.stderr || ''}`.slice(0, 4000);
    console.log(JSON.stringify({ decision: 'block', reason: output }));
  }
});
