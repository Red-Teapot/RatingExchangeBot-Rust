CREATE TABLE exchanges (
    id INTEGER PRIMARY KEY NOT NULL,
    guild INTEGER NOT NULL,
    jam_type TEXT NOT NULL,
    jam_link TEXT NOT NULL,
    slug TEXT NOT NULL,
    display_name TEXT NOT NULL,
    submission_channel INTEGER NOT NULL,

    UNIQUE (guild, slug)
);

CREATE TABLE exchange_rounds (
    id INTEGER PRIMARY KEY NOT NULL,
    exchange_id INTEGER NOT NULL,
    submissions_start_at TEXT NOT NULL,
    submissions_end_at TEXT NOT NULL,
    assignments_sent_at TEXT NOT NULL,
    games_per_member TEXT NOT NULL,
    state TEXT NOT NULL,

    CONSTRAINT fk_exchange
        FOREIGN KEY (exchange_id)
        REFERENCES exchanges(id)
        ON DELETE CASCADE
);

CREATE TABLE submissions (
    id INTEGER PRIMARY KEY NOT NULL,
    exchange_round_id INTEGER NOT NULL,
    link TEXT NOT NULL,
    submitter TEXT NOT NULL,
    submitted_at TEXT NOT NULL,

    CONSTRAINT fk_exchange_round
        FOREIGN KEY (exchange_round_id)
        REFERENCES exchange_rounds(id)
        ON DELETE CASCADE,

    UNIQUE (exchange_round_id, link)
);

CREATE TABLE played_games (
    id INTEGER PRIMARY KEY NOT NULL,
    link TEXT NOT NULL,
    member INTEGER NOT NULL,
    is_manual INTEGER NOT NULL,

    UNIQUE (link, member)
);
