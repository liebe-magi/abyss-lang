// @ts-check
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';

import vercel from '@astrojs/vercel';
import abyssGrammar from '../editors/code/syntaxes/abyss.tmLanguage.json' assert { type: 'json' };

const abyssLanguage = {
    ...abyssGrammar,
    name: 'abyss',
    scopeName: abyssGrammar.scopeName ?? 'source.abyss',
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
                    label: 'Overview',
                    items: [
                        { label: 'Welcome', slug: 'index' },
                        { label: 'Getting Started', slug: 'getting-started' },
                        { label: 'Roadmap', slug: 'roadmap' },
                    ],
                },
                {
                    label: 'Reference',
                    autogenerate: { directory: 'reference' },
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
});