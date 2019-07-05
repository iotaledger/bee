# bee-display

Printing relevant live data of Bee to the terminal.

## Documentation

To see the current documention for this crate with code examples, simply run:

```Bash
    $ cargo doc
    $ your_browser target/doc/bee_display/index.html
```

## Usage

```Rust
// Create a new Bee terminal display.
let mut display = Display::new();

// Clear the terminal
display.clear();

// Print a header section with relevant node live data.
display.header();

// Display a heartbeat animation. 
display.heartbeat();

// Shutdown the display.
display.close();
```

## Warning

This API is very likely to change frequently as long as Bee is in pre-1.0.0 stage of  development.

## Expected Features

* Bee logo
* Current tps
* Cpu utilization
* Transaction cache size
* Memory consumption
* Number of connected peers
* Node Id

## Contribution

Help to improve this crate for example by implementing one or more of the expected features.

## How this _will_ work eventually

In Bee everything will eventually be connected through EEE, which means that the `Display` type will host several `Entity`s that are subscribed to certain `Environment`s to gather all information it needs. 

For example to print the current transactions per second some `Entity` will join an `Environment` that gets updated each time a transaction has observed.