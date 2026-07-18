import { defineCollection, z } from 'astro:content';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';

export const collections = {
	docs: defineCollection({
		loader: docsLoader(),
		schema: docsSchema({
			extend: z.object({
				// 教材ページ用の拡張フィールド（プロジェクト情報ページ等では省略可）
				part: z.number().optional(),
				lesson: z.number().optional(),
				difficulty: z.enum(['basic', 'intermediate', 'advanced']).optional(),
				estimated_minutes: z.number().optional(),
				prerequisites: z.array(z.string()).optional(),
				hardware: z.array(z.string()).optional(),
				status: z
					.enum(['planned', 'outlined', 'drafted', 'reviewed', 'complete'])
					.optional(),
				code_status: z
					.enum([
						'none',
						'concept-only',
						'syntax-reviewed',
						'cargo-check-passed',
						'hardware-tested',
					])
					.optional(),
				verified_with: z.string().optional(),
				last_verified: z.string().optional(),
				sources: z.array(z.string()).optional(),
			}),
		}),
	}),
};
