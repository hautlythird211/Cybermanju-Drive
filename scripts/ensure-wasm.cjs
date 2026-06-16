const { existsSync } = require('fs');
const { execSync } = require('child_process');
if (!existsSync('node_modules/cybermanju-drive-wasm')) {
  execSync('npm run wasm:build-rust', { stdio: 'inherit' });
}
