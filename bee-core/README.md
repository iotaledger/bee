# bee-core

Crate that contains the Bee node itself and EEE model based messaging.

## What is EEE?

EEE stands for Environment-Entity-Effect. It is an subscription based communication protcol. It introduces the following abstractions:
* __Effect__: a type that represents a certain type of data which is sent between environments and entities.
* __Entity__: a type that can receive effects from environments it has joined, processes them, and then sends them to environments it affects. It can send effects by itself.
* __Environment__: a type that can receive and broadcast effects to entities, but not modify them.
* __Supervisor__: a central registry for all environments in the system, that can issue effects from a queue.

## Why does Bee need it?

EEE can be used to greatly decouple different parts of the node. As an example imagine you want write a `Display` extension for Bee to display live data from the node, but you don't want to tightly couple it, so that you can replace it quickly or remove it entirely to spare resources. Then all this extension has to do is subscribe to environments the Bee node provides, e.g. environment "TX". 

TODO