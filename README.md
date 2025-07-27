# Rating Exchange Bot

This is a Discord bot that runs the so-called rating exchanges:

1. When an exchange starts, members can submit their games to the bot until the exchange deadline.
2. When the exchange ends, the bot chooses a specific number of games to send to each participant. The number of games can be configured when creating an exchange.
3. The participants are supposed to play the games assigned by the bot and rate them. This is an honor-based system, the bot does not check whether the participant has rated a game.

## Assignments

The assignment algorithm is designed with the following goals in mind:
1. Don't assign participants' own submissions to them.
2. Don't assign a game that has been previously assigned to a specific participant (i.e., no duplicates).
3. Try to assign the games as evenly as possible (i.e., avoid assigning one game to all participants and not assigning another game to anyone at all).

Currently, this is implemented using the [Dinic's algorithm](https://en.wikipedia.org/wiki/Dinic's_algorithm) for computing a maximum flow in a flow network. The source is connected to a set of nodes corresponding to the participants. The sink is connected to a set of nodes representing the games. Then, for each pair of nodes `(participant, game)` an edge is added if it's possible to assign the game to the participant (i.e., it hasn't been assigned previously). The maximum number of games per participant is controlled via the capacity of the edges coming from the source and going to the sink.

## Checking the Results

The bot does not check whether a participant rates their assignments. Besides the technical reason (getting that data is complicated and is a privacy risk), there are moral concerns: forcing people to rate a game may lead to dishonest ratings. Not having ratings in that case seems to be the lesser evil.

# Development

This is a Rust project, so you need [cargo](https://github.com/rust-lang/cargo). To work with the database, you might also need [sqlx-cli](https://crates.io/crates/sqlx-cli). If you have Nix, you can run `nix develop` to get into a development shell that has the dependencies installed.

## Setting Up the Environment

The bot uses environment variables for configuration. To set these up, copy the `.env.example` file, rename the copy into `.env` and fill in the correct values. At the bare minimum, you need to set the Discord bot token and enable registering commands globally or in specific guilds (the latter is preferred for testing because changes to global commands may take a long time to propagate).

## Creating a Test Database

The bot uses an SQLite database to store its data. SQLx (the crate the bot uses to interact with the database) performs compile-time query checks. For that to work, or if you want to run the bot, you need to set up the database:

```sh
sqlx database setup
```

## Deployment

This bot is distributed as a Docker image that can be found [here](https://github.com/Red-Teapot/RatingExchangeBot-Rust/pkgs/container/rating-exchange-bot).

# License

Rating Exchange Bot is free, open source and permissively licensed! All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option. This means you can select the license you prefer! This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.
