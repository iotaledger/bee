/**
 * * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

module.exports = {
  mySidebar: [
    {
      type: 'doc',
      id: 'welcome',
      label: 'Welcome'
    },
    {
      type: 'doc',
      id: 'getting_started/getting_started',
      label: 'Getting Started',
    },
    {
      type: 'category',
      label: 'How To',
      items: [
        {
          type: 'doc',
          id: 'how_tos/setup_a_node',
          label: 'Setup a Node',
        },
        {
          type: 'doc',
          id: 'how_tos/docker',
          label: 'Using Docker',
        },
      ],
    },
    {
      type: 'category',
      label: 'Explanations',
      items: [
        {
          type: 'doc',
          id: 'explanations/nodes_101',
          label: 'Nodes 101',
        },
        {
          type: 'doc',
          id: 'explanations/security_101',
          label: 'Security 101',
        },
      ],
    },
    {
      type: 'category',
      label: 'References',
      items: [
        {
          type: 'doc',
          id: 'references/configuration',
          label: 'Configuration',
        },
        {
          type: 'doc',
          id: 'references/api_reference',
          label: 'API Reference',
        },
        {
          type: 'doc',
          id: 'references/crate_overview',
          label: 'Crate Overview',
        },
      ],
    },
    {
      type: 'doc',
      id: 'contribute',
      label: 'Contribute',
    },
    {
      type: 'doc',
      id: 'security_vulnerabilities',
      label: 'Security Vulnerabilities',
    },
    {
      type: 'doc',
      id: 'code_of_conduct',
      label: 'Code of Conduct',
    },
  ]
};
