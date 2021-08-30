import React, {useState} from 'react'
import {useHistory} from "react-router-dom";
import clsx from 'clsx';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useThemeContext from '@theme/hooks/useThemeContext';
import './styles.css'


function Action({title, link, description}) {
    let [hovering, setHovering] = useState(false);
    let history = useHistory();

    const handleClick = (e) => {
        e.preventDefault();
        history.push(link);
    }
    return (
        <div className='col col--4 margin-vert--md'>
            <div
                className='action padding--lg'
                onClick={handleClick}
                onMouseOver={() => setHovering(true)}
                onMouseOut={() => setHovering(false)}
            >
                <div className='action__header'>
                    <span className='action__title'>{title}</span>
                    <a href={link} className='action__button'>
                        <span className='action__icon material-icons'>
                          navigate_next
                        </span>
                    </a>
                </div>
                <div className={clsx(
                    'headline-stick',
                    {
                        'size-m': hovering,
                        'size-s': !hovering
                    }
                )}/>
                <div className='action__description'>
                    {description}
                </div>
            </div>
        </div>
    );
}

function LandingPageHeader({actionList, title, tagline}) {
    const {isDarkTheme} = useThemeContext();

    return (
        <header className='header padding-vert--xl'>
            <div className='title margin-horiz--sm'>
                <img className='title__image'
                     src={isDarkTheme ? useBaseUrl('/img/globe_dark.svg') : useBaseUrl('/img/globe_light.svg')} alt="IOTA Wiki"/>
                <div>
                    <h1 className='title__text'>{title}</h1>
                    <span className='title__subtext grey'>{tagline}</span>
                </div>
            </div>
            <div className='margin-top--xl'>
                <div className='section-header text--center margin-bottom--sm'>Get started, right away</div>
                <div className='actionlist row'>
                    {actionList.map((props, idx) => (
                        <Action key={idx} {...props} />
                    ))}
                </div>
            </div>
        </header>
    )
}

export default LandingPageHeader
