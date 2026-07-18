// docs/project/*.md を site/src/content/docs/project/ へコピーする。
// リポジトリ内の相対リンク（./versions.md 等）はサイトの絶対パスへ書き換える。
// サイトビルド前に自動実行される（site/package.json の prebuild）。
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const srcDir = join(root, 'docs', 'project');
const dstDir = join(root, 'site', 'src', 'content', 'docs', 'project');
const BASE = '/embassy-esp32-c6';

mkdirSync(dstDir, { recursive: true });

const files = readdirSync(srcDir).filter((f) => f.endsWith('.md'));
for (const f of files) {
	let text = readFileSync(join(srcDir, f), 'utf8');
	// ./name.md → /embassy-esp32-c6/project/name/
	text = text.replace(/\]\(\.\/([\w-]+)\.md\)/g, `](${BASE}/project/$1/)`);
	writeFileSync(join(dstDir, f), text);
}
console.log(`[sync-project-docs] copied ${files.length} files to site/src/content/docs/project/`);
