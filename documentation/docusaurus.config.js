const path = require('path');

module.exports = {
    plugins: [
        [
            '@docusaurus/plugin-content-docs',
            {
                id: 'bee-develop',
                path: path.resolve(__dirname, 'docs'),
                routeBasePath: 'bee',
                sidebarPath: path.resolve(__dirname, 'sidebars.js'),
                editUrl: 'https://github.com/iotaledger/bee/edit/shimmer-develop/documentation',
                remarkPlugins: [require('remark-code-import'), require('remark-import-partial')],
                versions: {
                    current: {
                        label: 'Develop',
                        path: 'develop',
                        badge: true
                    },
                },
            }
        ],
    ],
    staticDirectories: [path.resolve(__dirname, 'static')],
};
