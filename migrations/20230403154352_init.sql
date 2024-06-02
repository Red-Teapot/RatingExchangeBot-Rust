CREATE TABLE exchanges (
    id INTEGER PRIMARY KEY NOT NULL,

    guild INTEGER NOT NULL,
    channel INTEGER NOT NULL,

    jam_type TEXT NOT NULL,
    jam_link TEXT NOT NULL,

    slug TEXT NOT NULL,
    display_name TEXT NOT NULL,

    state TEXT NOT NULL,
    submissions_start DATETIME NOT NULL,
    submissions_end DATETIME NOT NULL,

    UNIQUE (guild, slug)
);

CREATE TABLE submissions (
    id INTEGER PRIMARY KEY NOT NULL,
    exchange_id INTEGER NOT NULL,
    link TEXT NOT NULL,
    submitter TEXT NOT NULL,
    submitted_at TEXT NOT NULL,

    CONSTRAINT fk_submissions_exchange_id
        FOREIGN KEY (exchange_id)
        REFERENCES exchanges(id)
        ON DELETE CASCADE,

    UNIQUE (exchange_id, link)
);

CREATE TABLE played_games (
    id INTEGER PRIMARY KEY NOT NULL,
    link TEXT NOT NULL,
    member INTEGER NOT NULL,
    is_manual INTEGER NOT NULL,

    UNIQUE (link, member)
);
