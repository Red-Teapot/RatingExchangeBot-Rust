CREATE TABLE exchanges (
    id SERIAL PRIMARY KEY,
    guild BIGINT,
    slug VARCHAR(32),
    display_name VARCHAR(32),
    submission_channel BIGINT
);

CREATE TABLE exchange_rounds (
    id SERIAL PRIMARY KEY,
    exchange_id BIGINT, 
    submissions_start_at TIMESTAMPTZ,
    submissions_end_at TIMESTAMPTZ,
    assignments_sent_at TIMESTAMPTZ,
    state INT,

    CONSTRAINT fk_exchange 
        FOREIGN KEY (exchange_id) 
        REFERENCES exchanges(id)
        ON DELETE CASCADE
);

CREATE TABLE submissions (
    id SERIAL PRIMARY KEY,
    exchange_round_id BIGINT,
    link VARCHAR(64),
    submitter BIGINT,
    submitted_at TIMESTAMPTZ,

    CONSTRAINT fk_exchange_round 
        FOREIGN KEY (exchange_round_id) 
        REFERENCES exchange_rounds(id)
        ON DELETE CASCADE,

    UNIQUE (exchange_round_id, link)
);

CREATE TABLE played_games (
    id SERIAL PRIMARY KEY,
    link VARCHAR(64),
    member INTEGER,
    is_manual BOOLEAN,

    UNIQUE (link, member)
);
