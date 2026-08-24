import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";
import tailwind from "@tailwindcss/postcss";
import autoprefixer from "autoprefixer";

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: "Haste Health",
  tagline:
    "The Headless EHR & FHIR Clinical Data Repository, Built for AI Agents",
  favicon: "/img/favicon.ico",
  trailingSlash: false,

  headTags: [
    {
      tagName: "meta",
      attributes: {
        name: "algolia-site-verification",
        content: "2EFDB046F281A382",
      },
    },
    // Site-wide Organization structured data (schema.org). Individual pages
    // should NOT repeat this block — see generate.mjs for the per-resource
    // structured data used on generated FHIR reference pages instead.
    {
      tagName: "script",
      attributes: { type: "application/ld+json" },
      innerHTML: JSON.stringify({
        "@context": "https://schema.org",
        "@type": "Organization",
        name: "Haste Health",
        url: "https://haste.health",
        logo: "https://haste.health/img/logo.svg",
        description:
          "Haste Health is an open-source, self-hosted headless EHR and FHIR R4 clinical data repository that normalizes Epic, Cerner, HL7v2, and other healthcare data sources into one API for apps and AI agents.",
        sameAs: ["https://github.com/hastehealth/hastehealth"],
      }),
    },
  ],

  onBrokenLinks: "throw",

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: "https://haste.health",
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: "/",

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: "hastehealth", // Usually your GitHub org/user name.
  projectName: "hastehealth", // Usually your repo name.

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  markdown: {
    mermaid: true,
  },
  themes: ["@docusaurus/theme-mermaid"],

  plugins: [
    async function myPlugin(context, options) {
      return {
        name: "docusaurus-tailwindcss",
        configurePostCss(postcssOptions) {
          // Appends TailwindCSS and AutoPrefixer.

          postcssOptions.plugins.push(tailwind, autoprefixer);
          return postcssOptions;
        },
      };
    },
  ],

  presets: [
    [
      "@docusaurus/preset-classic",
      {
        sitemap: {
          lastmod: "date",
          changefreq: "weekly",
          priority: 0.5,
          filename: "sitemap.xml",
        },
        docs: {
          sidebarPath: "./sidebars.ts",
          editUrl:
            "https://github.com/hastehealth/hastehealth/tree/main/frontend/packages/website/",
        },
        gtag: {
          trackingID: "G-HH00J38YZ5",
        },
        blog: {
          authorsBasePath: "team",
          blogSidebarTitle: "All posts",
          blogSidebarCount: "ALL",

          showReadingTime: true,
          feedOptions: {
            type: ["rss", "atom"],
            xslt: true,
          },
          // Useful options to enforce blogging best practices
          onInlineTags: "warn",
          onInlineAuthors: "warn",
          onUntruncatedBlogPosts: "warn",
        },
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    // Default social card (og:image / twitter:image) for any page that
    // doesn't set its own `image` in Layout/frontmatter. TODO: swap for a
    // dedicated 1200x630 branded social card — this is a product screenshot
    // used as an interim placeholder because it's the closest asset on hand
    // to the recommended aspect ratio.
    image: "img/admin_app.png",
    // Global <meta> tags injected on every page. Per-page title/description
    // (set via Layout props or frontmatter) always take precedence over
    // these defaults, but keywords/robots/twitter card have no per-page
    // equivalent in Docusaurus, so they're set once here.
    metadata: [
      {
        name: "keywords",
        content:
          "headless EHR, FHIR server, FHIR clinical data repository, FHIR CDR, HL7 FHIR R4 API, SMART on FHIR, OAuth2 provider, healthcare interoperability, MCP AI agents, self-hosted EHR, open source FHIR server",
      },
      { name: "robots", content: "index, follow" },
      { name: "twitter:card", content: "summary_large_image" },
      { property: "og:site_name", content: "Haste Health" },
      { property: "og:type", content: "website" },
    ],
    colorMode: {
      defaultMode: "light",
      disableSwitch: true,
      respectPrefersColorScheme: false,
    },
    navbar: {
      // title: "haste.health",
      logo: {
        alt: "haste.health",
        src: "img/logo_text.svg",
        style: {
          width: "128px",
        },
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "documentationSidebar",
          position: "left",
          label: "Documentation",
        },
        { to: "/blog", label: "Blog", position: "left" },
        {
          to: "https://calendly.com/rp-haste/book-a-demo",
          position: "right",
          label: "Book a Demo",
          className: "signup-link",
        },
        {
          type: "search",
          position: "right",
        },
        {
          href: "https://github.com/hastehealth/hastehealth",
          "aria-label": "GitHub repository",
          className: "header-github-link",
          position: "right",
        },
        {
          to: "https://api.haste.health/auth/login",
          label: "Log in",
          position: "right",
          className: "signup-link",
        },
        {
          to: "https://api.haste.health/auth/signup",
          label: "Start for free",
          position: "right",
          className:
            "text-white bg-brand-800 hover:bg-brand-900 rounded-4xl px-4 py-2",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Docs",
          items: [
            {
              label: "Getting Started",
              to: "/docs/getting_started/quick_start",
            },
            {
              label: "Core Concepts",
              to: "/docs/core_concepts/platform_architecture",
            },
            {
              label: "API Reference",
              to: "/docs/api/rest_api/fhir/intro",
            },
          ],
        },
        {
          title: "Community",
          items: [
            {
              label: "Contact",
              to: "/contact",
            },
            {
              label: "Report an Issue",
              href: "https://github.com/hastehealth/hastehealth/issues",
            },
            {
              label: "Request a Feature",
              href: "/docs/rfc/how_to_submit",
            },
            // {
            //   label: "Stack Overflow",
            //   href: "https://stackoverflow.com/questions/tagged/docusaurus",
            // },
            // {
            //   label: "Discord",
            //   href: "https://discordapp.com/invite/docusaurus",
            // },
            // {
            //   label: "X",
            //   href: "https://x.com/docusaurus",
            // },
          ],
        },
        {
          title: "More",
          items: [
            {
              label: "Blog",
              to: "/blog",
            },
            {
              label: "GitHub",
              href: "https://github.com/hastehealth/hastehealth",
            },
          ],
        },
      ],

      copyright: `Copyright © ${new Date().getFullYear()} Haste Health, Inc. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
    algolia: {
      // The application ID provided by Algolia
      appId: "9M3PZB2S4M",

      // Public API key: it is safe to commit it
      apiKey: "06ecebc0265bb31186be933a6c982863",

      indexName: "Haste Health",

      // Optional: see doc section below
      contextualSearch: true,

      // Optional: Specify domains where the navigation should occur through window.location instead on history.push. Useful when our Algolia config crawls multiple documentation sites and we want to navigate with window.location.href to them.
      externalUrlRegex: "external\\.com|domain\\.com",

      // // Optional: Replace parts of the item URLs from Algolia. Useful when using the same search index for multiple deployments using a different baseUrl. You can use regexp or string in the `from` param. For example: localhost:3000 vs myCompany.com/docs
      // replaceSearchResultPathname: {
      //   from: "/docs/", // or as RegExp: /\/docs\//
      //   to: "/",
      // },

      // Optional: Algolia search parameters
      searchParameters: {},

      // Optional: path for search page that enabled by default (`false` to disable it)
      searchPagePath: "search",

      // Optional: whether the insights feature is enabled or not on Docsearch (`false` by default)
      insights: false,

      // Optional: whether you want to use the new Ask AI feature (undefined by default)
      // askAi: "YOUR_ALGOLIA_ASK_AI_ASSISTANT_ID",

      //... other Algolia params
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
