// @ts-check
import { defineConfig } from 'astro/config'
import starlight from '@astrojs/starlight'
import starlightLinksValidator from 'starlight-links-validator'
import { LOCALES, t } from './locales.mjs'

// A project page of the organisation's GitHub Pages: `oxidestack.github.io` is shared by every project of the
// organisation, so this site lives under `/libphonenumber/`, and every internal link carries that base. The staging
// script writes the links with it (scripts/stage.mjs, BASE), and the validator below fails a link without it.
export default defineConfig({
  site: 'https://oxidestack.github.io',
  base: process.env.SITE_BASE || '/libphonenumber',
  trailingSlash: 'always',
  integrations: [
    starlight({
      title: 'libphonenumber',
      // PLACEHOLDER: no logo or favicon yet; they go here
      // (`logo: { src: './src/assets/<mark>.svg' }`, `favicon`) once the artwork exists.
      description: 'An unofficial Rust port of the parts of Google\'s libphonenumber 9.0.41 (Java) that Datafaker reaches: metadata, parsing, validation, number types and formatting. Not affiliated with Google.',
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/oxidestack/libphonenumber' }],
      // Each staged page sets its own editUrl to its source file: the default would point at the staged copy,
      // which the next build deletes.
      editLink: { baseUrl: 'https://github.com/oxidestack/libphonenumber/edit/main/' },
      // English at the root, every other language under its directory (locales.mjs, site/README.md "Languages").
      defaultLocale: 'root',
      locales: LOCALES,
      sidebar: [
        { ...t('Introduction', 'Введение', '简介', 'Introducción'), slug: 'index' },
        { ...t('Upgrading', 'Обновление', '升级', 'Actualización'), slug: 'upgrading' },
      ],
      plugins: [
        // A broken internal link or anchor fails the build: every page is staged from a file of the repository, so a
        // link that stops resolving is a heading or a file renamed there, and the build is where to hear of it. A
        // translation's link to a page not translated yet is to that page's English fallback in its language.
        starlightLinksValidator({ errorOnRelativeLinks: true, errorOnInvalidHashes: true, errorOnFallbackPages: false }),
      ],
    }),
  ],
})
