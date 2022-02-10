---
description: This section provides a checklist of steps for running a reliable and secure node.
image: /img/logo/bee_logo.png
keywords:
- nodes
- reverse proxy
- ports
- API
- requests
- SSH
---
# Security 101

This section explains how and why node security is important while also providing a checklist of steps for running a reliable and secure node.

:::info
Servers that are reachable from the Internet are a constant target from security challengers. Please, make sure you follow the minimum security essentials summarized in this article.
:::

## Securing Your Device
The security of the device that is running your node is of the utmost importance to stop attackers from gaining access to the node.

Before running a node on your device, consider:
* [Securing SSH logins](#securing-ssh-logins).
* [Blocking unnecessary ports](#blocking-unnecessary-ports).

### Securing SSH logins
If you log into your device through SSH, you should take measures to protect it from unauthorized access. Many guides and resources have been provided on this subject, including: 

- [10 Steps to Secure Open SSH](https://blog.devolutions.net/2017/04/10-steps-to-secure-open-ssh). 
- [Fail2ban](https://www.fail2ban.org/wiki/index.php/Main_Page)
    - Here, you can leverage tools to improve your node's security

### Blocking Unnecessary Ports
Attackers can abuse any open ports on your device. Closing all of your ports except those that are in use helps secure your device from attacks on unused or open ports.

You can use a firewall to accomplish this, and all operating systems include firewall options. By having a firewall in place, you can completely block unused and unnecessary ports.

On cloud platforms such as AWS, Azure, or GCP, you can even block ports on VPS networking settings.

## Deciding Whether to Enable Remote Proof of Work
When you are configuring your node, you have the option to allow it to do Proof of Work (PoW). If you enable this feature, clients can ask your node to do remote PoW.

PoW takes time and uses your node's computational power. So consider enabling it according to your infrastructure.

## Load Balancing
If you run more than one node, it's a good practice to make sure that you distribute the API requests among all of them.

To evenly distribute the API requests among all your nodes, you can run a reverse proxy server that will act as a load balancer ([HAProxy](http://www.haproxy.org/), [Traefik](https://traefik.io/), [Nginx](https://www.nginx.com/), [Apache](https://www.apache.org/), etc.). This way, you can have one domain name for your reverse proxy server that all nodes will send their API calls to. On the backend, the nodes with the most spare computational power will process the request and return the response.

Since broadcasted messages are atomic and nodes provide restful API to communicate, you will not need sticky sessions or similar technologies.

## Reverse Proxy
We recommend that you use a reverse proxy in front of a node is even if you are deploying a single node. Using a reverse proxy adds a security layer that can handle tasks such as:

- IP address filtering. 
- Abuse rate limiting. 
- SSL encrypting.
- Additional authorization layer.
