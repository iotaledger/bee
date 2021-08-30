import React from 'react';
import Layout from '@theme/Layout';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import LandingPageHeader from '@site/src/components/LandingPageHeader';

const actionList = [
    {
        title: 'Learn',
        link: 'welcome',
        description: 'Learn the Basics about the IOTA Bee node and how it works behind the scenes.'
    },
    {
        title: 'Build',
        link: 'setup_a_node',
        description: 'Follow our tutorial to run your own IOTA Bee node.'
    },
    {
        title: 'Participate',
        link: 'contribute/contribute',
        description: 'Do you want to be a part of the IOTA mission? Join the IOTA community.'
    },
];
const title = 'Bee';
const tagline = 'Official IOTA Bee Software';
export default function Home() {
    const {siteConfig} = useDocusaurusContext();

    return (
        <Layout title={`${siteConfig.title}`} description={`${siteConfig.tagline}`}>
            <LandingPageHeader actionList={actionList} title={title} tagline={tagline}/>
        </Layout>
    );
}
