# matrix-rust-idiotbot

a bot for [matrix](https://matrix.org) to let people know when they are being an idiot

no AI was involved, just human stupidity

some code taken from https://github.com/matrix-org/matrix-rust-sdk/tree/main/examples

## features

- `.idiot <user> <reason>` to add a new case (aka expose someone for being an idiot)
- `.stats` to get stats (most upvoted reports and biggest idiots)
- react to an existing report with 🔥 to upvote

## compilation
- install rust and cargo from https://rustup.rs/
- clone this repository
- run `cargo build --release`
- the binary will be in `target/release/matrix-rust-idiotbot`

## usage
- `target/release/matrix-rust-idiotbot <HOMESERVER_URL> <USERNAME> <PASSWORD>`
- the bot will autojoin rooms it is invited to and listen for commands

## todo

- record time when reports are created
- handle edits & deletions of idiot commands
- encrypted room support
- save idiot reports to disk or database
