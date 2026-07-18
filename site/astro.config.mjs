// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';

// https://astro.build/config
export default defineConfig({
	site: 'https://tomixrm.github.io',
	base: '/embassy-esp32-c6',
	integrations: [
		mermaid({
			theme: 'default',
			autoTheme: true,
		}),
		starlight({
			title: 'ESP32-C6 × Rust × Embassy 教科書',
			description:
				'ArduinoからステップアップしてESP32-C6をRustとEmbassyで動かす、中学生から読める日本語教材',
			defaultLocale: 'root',
			locales: {
				root: { label: '日本語', lang: 'ja' },
			},
			social: [
				{
					icon: 'github',
					label: 'GitHub',
					href: 'https://github.com/TomiXRM/embassy-esp32-c6',
				},
			],
			sidebar: [
				{ label: 'はじめに', slug: 'intro' },
				{
					label: '第1部 ESP32-C6と開発環境',
					items: [{ autogenerate: { directory: 'part01' } }],
					collapsed: true,
				},
				{
					label: '第2部 Rustの最初の一歩',
					items: [{ autogenerate: { directory: 'part02' } }],
					collapsed: true,
				},
				{
					label: '第3部 Rustらしいデータの扱い',
					items: [{ autogenerate: { directory: 'part03' } }],
					collapsed: true,
				},
				{
					label: '第4部 大きなプログラムの作り方',
					items: [{ autogenerate: { directory: 'part04' } }],
					collapsed: true,
				},
				{
					label: '第5部 組み込みRustの基礎',
					items: [{ autogenerate: { directory: 'part05' } }],
					collapsed: true,
				},
				{
					label: '第6部 GPIO・割り込み・時間',
					items: [{ autogenerate: { directory: 'part06' } }],
					collapsed: true,
				},
				{
					label: '第7部 アナログと波形制御',
					items: [{ autogenerate: { directory: 'part07' } }],
					collapsed: true,
				},
				{
					label: '第8部 UART・I2C・SPI・TWAI',
					items: [{ autogenerate: { directory: 'part08' } }],
					collapsed: true,
				},
				{
					label: '第9部 Embassyによる非同期処理',
					items: [{ autogenerate: { directory: 'part09' } }],
					collapsed: true,
				},
				{
					label: '第10部 Wi-Fiとネットワーク',
					items: [{ autogenerate: { directory: 'part10' } }],
					collapsed: true,
				},
				{
					label: '第11部 BLE・ESP-NOW・802.15.4',
					items: [{ autogenerate: { directory: 'part11' } }],
					collapsed: true,
				},
				{
					label: '第12部 実用設計と最終プロジェクト',
					items: [{ autogenerate: { directory: 'part12' } }],
					collapsed: true,
				},
				{
					label: '応用編 キーボードを作る視点',
					items: [{ autogenerate: { directory: 'keyboard' } }],
					collapsed: true,
				},
				{
					label: '応用編 センサ端末を作る視点',
					items: [{ autogenerate: { directory: 'sensor-node' } }],
					collapsed: true,
				},
				{
					label: '応用編 ロボットファームを読む',
					items: [{ autogenerate: { directory: 'robot' } }],
					collapsed: true,
				},
				{
					label: '応用編 ESP32-C6の深淵',
					items: [{ autogenerate: { directory: 'deep-dive' } }],
					collapsed: true,
				},
				{
					label: '付録',
					items: [{ autogenerate: { directory: 'appendix' } }],
					collapsed: true,
				},
				{
					label: 'プロジェクト情報',
					items: [{ autogenerate: { directory: 'project' } }],
					collapsed: true,
				},
			],
		}),
	],
});
