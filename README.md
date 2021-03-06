# snapmail-cli

Terminal User Interface and Command line interface app for [SnapMail](https://github.com/glassbeadsoftware/snapmail-release) from [Glass Bead Software](http://www.glassbead.com/).

CI and NIX configs are not set up for the moment.

## Commands

`````
Snapmail CLI

Command line interface for Snapmail DNA

USAGE:
    snapmail-cli <sid> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <sid>    Session ID. Corresponds to an unique config, network id and agent

SUBCOMMANDS:
    change            Modify the setup
    clear             Erase a session from disk
    chain             Print source-chain in terminal
    directory         Display all users part of the current network
    get-attachment    Extract an attachment from a mail
    get-handle        Get agent's current handle
    help              Prints this message or the help of the given subcommand(s)
    info              Display setup (conductor config, uid)
    list              List all mails received by this agent
    list-sessions     List sessions that have been setup on this computer
    listen            Launch an "always on" conductor that displays events & signals
    open              Read mail from mailbox (Will send an acknowledgement to mail author)
    ping              Check if a user is currently online
    pull              Query the DHT for all relevant data (handles, mailbox, ackbox)
    send              Send a mail to another agent    
    set-handle        Change agent's handle
    setup             Create agent and config
    status            Show a mail's state (Unsent, acknowledged...)
    resend            Check outbox and resend mails who do not have acknowledge status
`````

## Examples

##### Setup
`````
snapmail-cli alex setup testnet network -b https://bootstrap-staging.holo.host/ quic
`````
##### Send mail
`````
snapmail-cli alex send --to billy -m "hello world!" -s "First post" -a ../clover.jpg
`````
##### Open mail
`````
snapmail-cli billy list
snapmail-cli billy open uhCkk69Fu0YwACllB__HLWwN49vCVf8JIOfKDuBXjMjG5BWcH2Tq4
`````

# Snapmail-tui

`````
Snapmail TUI

Terminal user interface for Snapmail DNA

USAGE:
    snapmail-tui <sid>

FLAGS:
    -l, List available Session IDs
    -h, Prints help information
    -V, Prints version information

ARGS:
    <sid>    Session ID. Corresponds to an unique config, network id and agent
`````

## Usage Guide

First use the CLI to setup an agent.

Press keys corresponding to the highlighted letters to navigate. <br/>
'Q' Key to exit app.

### View Screen

Top bar information corresponds to: SessionId, NetworkId, Username, Number of connected peers in the network. <br/>
Up/Down Keys to select mail. <br/>
Press Enter key to make selected mail scrollable with Up/Down Keys. Esc key to go back. <br/>
Number keys are used to donwload attachments with corresponding index number. <br/>
Press Delete key to trash selected mail.

![screenshot-view](/sshots/snap-view.png)

### Write Screen

Tab key to toggle between edit blocks. <br/>
Up/Down keys to select a contact. <br/>
Enter to toggle contact send state (to, cc, bcc). <br/>
When no block is selected use Enter or Insert key to send mail. <br/>
Attachment must be a valid path on drive.

![screenshot-write](/sshots/snap-write.png)

### Edit Screen

Use the highlighted letters to select which setting to change. <br/>
Press Enter key to confirm settings change. <br/>
Press Esc key to cancel editing.<br/>

Bootstrap URL and Proxy URL currently not changeable. Redo setup with CLI or manually edit the conductor-config.yaml in your .config folder.

![screenshot-write](/sshots/snap-edit.png)

# Building

1. `./scripts/setup.bat`
3. `./scripts/rustify.bat`
4. `cargo build --release`
