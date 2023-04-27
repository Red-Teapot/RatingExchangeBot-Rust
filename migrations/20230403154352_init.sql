CREATE TABLE exchanges (
    id SERIAL PRIMARY KEY,
    guild BIGINT NOT NULL,
    jam_type INTEGER NOT NULL,
    jam_link VARCHAR(64) NOT NULL,
    slug VARCHAR(32) NOT NULL,
    display_name VARCHAR(32) NOT NULL,
    submission_channel BIGINT NOT NULL,

    UNIQUE (guild, slug)
);

CREATE TABLE exchange_rounds (
    id SERIAL PRIMARY KEY,
    exchange_id INT NOT NULL,
    submissions_start_at TIMESTAMP NOT NULL,
    submissions_end_at TIMESTAMP NOT NULL,
    assignments_sent_at TIMESTAMP NOT NULL,
    games_per_member INTEGER NOT NULL,
    state INT NOT NULL,

    CONSTRAINT fk_exchange
        FOREIGN KEY (exchange_id)
        REFERENCES exchanges(id)
        ON DELETE CASCADE
);

CREATE TABLE submissions (
    id SERIAL PRIMARY KEY,
    exchange_round_id INT NOT NULL,
    link VARCHAR(64) NOT NULL,
    submitter BIGINT NOT NULL,
    submitted_at TIMESTAMP NOT NULL,

    CONSTRAINT fk_exchange_round
        FOREIGN KEY (exchange_round_id)
        REFERENCES exchange_rounds(id)
        ON DELETE CASCADE,

    UNIQUE (exchange_round_id, link)
);

CREATE TABLE played_games (
    id SERIAL PRIMARY KEY,
    link VARCHAR(64) NOT NULL,
    member BIGINT NOT NULL,
    is_manual BOOLEAN NOT NULL,

    UNIQUE (link, member)
);
