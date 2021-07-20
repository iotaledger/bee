const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').DocusaurusConfig} */
module.exports = {
  title: 'Bee',
  tagline: 'Official IOTA Bee Software',
  url: 'https://bee.docs.iota.org/',
  baseUrl: '/',
  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',
  favicon: '/img/logo/favicon.ico',
  organizationName: 'iotaledger', // Usually your GitHub org/user name.
  projectName: 'Bee', // Usually your repo name.
  stylesheets: [
    'https://fonts.googleapis.com/css?family=Material+Icons',
  ],
  themeConfig: {
    colorMode: {
          defaultMode: "dark",
          },
    navbar: {
      title: 'Bee',
      logo: {
        alt: 'IOTA',
        src: 'img/logo/Logo_Swirl_Dark.png',
      },
      items: [
        {
          type: 'doc',
          docId: 'welcome',
          position: 'left',
          label: 'Documentation',
        },
//        {to: '/blog', label: 'Blog', position: 'left'},
        {
          href: 'https://github.com/iotaledger/bee',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
      footer: {
      style: 'dark',
      links: [
        {
          title: 'Documentation',
          items: [
            {
              label: 'Welcome',
              to: '/welcome',
            },
            {
              label: 'Getting Started',
              to: '/getting_started/getting_started',
            },
            {
              label: 'Configuration',
              to: '/configuration',
            },
            {
              label: 'Set Up a Node',
              to: '/setup_a_node',
            },
            {
              label: 'API Reference',
              to: '/api_reference',
            },
            {
              label: 'Contribute',
              to: '/contribute/contribute',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'Discord',
              href: 'https://discord.iota.org/',
            },
          ],
        },
        {
          title: 'Contribute',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/iotaledger/bee',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} IOTA Foundation, Built with Docusaurus.`,
    },
    prism: {
        additionalLanguages: ['rust'],
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
    },
  },
  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          routeBasePath:'/',
          // Please change this to your repo.
          editUrl:
            'https://github.com/iotaledger/bee/tree/main/docs',
        },
        theme: {
          customCss: require.resolve('./src/css/iota.css'),
        },
      },
    ],
  ],

  plugins: [
    [
      'docusaurus-plugin-includes',
      {
        sharedFolders: [
          { source: '../.github/', target: '.github'},
          { source: '../', target:'README.md'}
        ],
      },
    ],
  ],
};
