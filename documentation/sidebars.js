/**
 * * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

module.exports = {
  mySidebar: [{
      type: 'doc',
      id: 'welcome',
      label: 'Welcome'
    },
    {
      type: 'category',
      label: 'Getting Started',
      items: [{
        type: 'doc',
        id: 'getting_started/getting_started',
        label: 'Getting Started',
      }, {
        type: 'doc',
        id: 'getting_started/nodes_101',
        label: 'Nodes 101',
      }, {
        type: 'doc',
        id: 'getting_started/security_101',
        label: 'Security 101',
      }, {
        type: 'doc',
        id: 'getting_started/docker',
        label: 'Using Docker',
      }, ],
    },
    {
      type: 'doc',
      id: 'configuration',
      label: 'Configuration',
    },
    {
      type: 'doc',
      id: 'setup_a_node',
      label: 'Setup a Node',
    },
    {
      type: 'doc',
      id: 'crate_overview',
      label: 'Crate Overview',
    },
    {
      type: 'doc',
      id: 'api_reference',
      label: 'API Reference',
    },
    {
      type: 'category',
      label: 'Contribute',
      items: [{
        type: 'doc',
        id: 'contribute/contribute',
        label: 'Contribute',
      }, {
        type: 'doc',
        id: 'contribute/security_vulnerabilities',
        label: 'Security Vulnerabilities',
      }, {
        type: 'doc',
        id: 'contribute/code_of_conduct',
        label: 'Code of Conduct',
      }, ]
    }
  ]
};
