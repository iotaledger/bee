const path = require('path');

module.exports = {
    plugins: [
        [
            '@docusaurus/plugin-content-docs',
            {
                id: 'bee',
                path: path.resolve(__dirname, 'docs'),
                routeBasePath: 'bee',
                sidebarPath: path.resolve(__dirname, 'sidebars.js'),
                editUrl: 'https://github.com/iotaledger/bee/edit/production/documentation',
                remarkPlugins: [require('remark-code-import'), require('remark-import-partial')],
            }
        ],
    ],
    staticDirectories: [path.resolve(__dirname, 'static')],
};
