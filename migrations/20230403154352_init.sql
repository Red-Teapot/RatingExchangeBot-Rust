CREATE TABLE exchanges (
    id INTEGER PRIMARY KEY NOT NULL,

    guild INTEGER NOT NULL,
    channel INTEGER NOT NULL,

    jam_type TEXT NOT NULL,
    jam_link TEXT NOT NULL,

    slug TEXT NOT NULL,
    display_name TEXT NOT NULL,

    state TEXT NOT NULL,
    submissions_start TEXT NOT NULL,
    submissions_end TEXT NOT NULL,

    games_per_member INTEGER NOT NULL CHECK(games_per_member > 0),

    CONSTRAINT uniq_guild_slug UNIQUE (guild, slug)
) STRICT;

CREATE TABLE submissions (
    id INTEGER PRIMARY KEY NOT NULL,
    exchange_id INTEGER NOT NULL,
    link TEXT NOT NULL,
    submitter INTEGER NOT NULL,
    submitted_at TEXT NOT NULL,

    CONSTRAINT fk_exchange_id
        FOREIGN KEY (exchange_id)
        REFERENCES exchanges(id)
        ON DELETE CASCADE,

    CONSTRAINT uniq_exchange_id_link UNIQUE (exchange_id, link),
    CONSTRAINT uniq_exchange_id_submitter UNIQUE (exchange_id, submitter)
) STRICT;

CREATE TABLE played_games (
    id INTEGER PRIMARY KEY NOT NULL,
    member INTEGER NOT NULL,
    link TEXT NOT NULL,
    is_manual INTEGER NOT NULL,

    CONSTRAINT uniq_link_member UNIQUE (link, member)
) STRICT;
