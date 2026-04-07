import { defineConfig } from "vitepress";

export default defineConfig({
  title: "broken-latch",
  description: "Developer documentation for the broken-latch overlay platform",

  themeConfig: {
    nav: [
      { text: "Home", link: "/" },
      { text: "Getting Started", link: "/getting-started/installation" },
      { text: "API Reference", link: "/api/" },
      { text: "Guides", link: "/guides/building-a-widget" },
      { text: "Examples", link: "/examples/minimal-widget" },
    ],

    sidebar: {
      "/getting-started/": [
        {
          text: "Getting Started",
          items: [
            { text: "Installation", link: "/getting-started/installation" },
            { text: "Your First App", link: "/getting-started/your-first-app" },
            { text: "App Manifest", link: "/getting-started/manifest" },
            { text: "Local Development", link: "/getting-started/local-development" },
          ],
        },
      ],

      "/api/": [
        {
          text: "API Reference",
          items: [
            { text: "Overview", link: "/api/" },
            { text: "Initialization", link: "/api/initialization" },
            { text: "Game Lifecycle", link: "/api/game-lifecycle" },
            { text: "Windows", link: "/api/windows" },
            { text: "Hotkeys", link: "/api/hotkeys" },
            { text: "Storage", link: "/api/storage" },
            { text: "Notifications", link: "/api/notifications" },
            { text: "Messaging", link: "/api/messaging" },
            { text: "Platform", link: "/api/platform" },
          ],
        },
      ],

      "/guides/": [
        {
          text: "Guides",
          items: [
            { text: "Building a Widget", link: "/guides/building-a-widget" },
            { text: "Game Phase Tracking", link: "/guides/game-phase-tracking" },
            { text: "Persistent Storage", link: "/guides/persistent-storage" },
            { text: "Backend Integration", link: "/guides/backend-integration" },
            { text: "Permissions", link: "/guides/permissions" },
            { text: "Debugging", link: "/guides/debugging" },
          ],
        },
      ],

      "/examples/": [
        {
          text: "Examples",
          items: [
            { text: "Minimal Widget", link: "/examples/minimal-widget" },
            { text: "Multi-Panel App", link: "/examples/multi-panel-app" },
            { text: "Hotkey Toggle", link: "/examples/hotkey-toggle" },
            { text: "Webhook Backend", link: "/examples/webhook-backend" },
          ],
        },
      ],

      "/reference/": [
        {
          text: "Reference",
          items: [
            { text: "Manifest Schema", link: "/reference/manifest-schema" },
            { text: "Permissions", link: "/reference/permissions" },
            { text: "Game Phases", link: "/reference/game-phases" },
            { text: "TypeScript Types", link: "/reference/typescript-types" },
          ],
        },
      ],
    },

    socialLinks: [
      { icon: "github", link: "https://github.com/broken-latch/platform" },
    ],

    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2026 broken-latch",
    },

    search: {
      provider: "local",
    },
  },
});
