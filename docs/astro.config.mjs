// @ts-check
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';

import vercel from '@astrojs/vercel';
import abyssGrammar from '../editors/code/syntaxes/abyss.tmLanguage.json' with { type: 'json' };

const abyssLanguage = {
    ...abyssGrammar,
    name: 'abyss',
    scopeName: abyssGrammar.scopeName,
    aliases: ['abyss-lang'],
};

// https://astro.build/config
export default defineConfig({
    site: 'https://abyss-lang.dev',
    integrations: [
        starlight({
            title: 'AbySS',
            description: 'The spellbinding guide to AbySS syntax, tooling, and workflows.',
            social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/liebe-magi/abyss-lang' }],
            sidebar: [
                {
                    label: 'Start Here',
                    items: [
                        { label: 'Welcome', slug: 'index' },
                        { label: 'Getting Started', slug: 'getting-started' },
                        { label: 'Playground', slug: 'playground' },
                    ],
                },
                {
                    label: 'The Grimoire',
                    items: [
                        { label: 'Basic Syntax', slug: 'reference/basic-syntax' },
                        { label: 'Variables', slug: 'reference/variables' },
                        { label: 'Data Types', slug: 'reference/types' },
                        { label: 'Type Casting', slug: 'reference/type-casting' },
                        { label: 'Input & Output', slug: 'reference/input-output' },
                        { label: 'Control Flow: Conditionals', slug: 'reference/conditionals' },
                        { label: 'Control Flow: Pattern Matching', slug: 'reference/pattern-matching' },
                        { label: 'Control Flow: Loops', slug: 'reference/loops' },
                        { label: 'Functions', slug: 'reference/functions' },
                        { label: 'Error Handling', slug: 'reference/error-handling' },
                        { label: 'Collections', slug: 'reference/collections' },
                        { label: 'Artifacts', slug: 'reference/artifacts' },
                    ],
                },
                {
                    label: 'Project',
                    items: [
                        { label: 'Roadmap', slug: 'roadmap' },
                    ],
                },
            ],
            expressiveCode: {
                shiki: {
                    langs: [
                        abyssLanguage,
                    ],
                },
            },
        }),
    ],

    adapter: vercel(),

    vite: {
        build: {
            rollupOptions: {
                // The Playground component dynamically imports the
                // wasm-pack output via `/wasm/abyss_wasm.js`. That URL is
                // served as a static asset at runtime (the file lives
                // under `docs/public/wasm/`), so Vite/Rollup must not try
                // to resolve it during the docs-site bundle step.
                external: [/^\/wasm\//],
            },
        },
    },
});