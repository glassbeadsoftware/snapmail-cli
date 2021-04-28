# snapmail-cli

Command line interface app for [SnapMail](https://github.com/glassbeadsoftware/snapmail-release) from [Glass Bead Software](http://www.glassbead.com/).

CI and NIX configs are not set up for the moment.

## Commands

`````
Snapmail CLI

Interface for Snapmail DNA 

USAGE:
    snapmail-cli <sid> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help           Prints this message or the help of the given subcommand(s)
    setup          Create agent and config
    clear          Erase agent and config
    info           Display setup (conductor config...)
    change         Change handle / config
    directory      Display all users part of the current network
    ping           Check if a user is online
    pull           Query the DHT for all relevant data (handles, mailbox, ackbox)
    send           Send a mail
    list           Display all mails (with filtering)
    open           Read mailbox
    get-attachment Extract an attachment from a mail
    listen         Listen to network events
`````

## Sub-commands
### Setup

`````
Snapmail CLI

Create agent and config

USAGE:
    snapmail-cli <sid> setup [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information

OPTIONS:
    -mdns           Use MDNS instead of bootstrap server

SUBCOMMANDS:
    network
`````

`````
snapmail-cli setup --name toto --mdns
`````

## Building

FIXME

## Testing
FIXME
## Running with UI
FIXME
