CREATE TABLE assignments (
    id INTEGER PRIMARY KEY NOT NULL,
    exchange_id INTEGER NOT NULL,
    submission_id INTEGER NOT NULL,
    submitter INTEGER NOT NULL,

    CONSTRAINT fk_submission_id
        FOREIGN KEY (submission_id)
        REFERENCES submissions(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_exchange_id
        FOREIGN KEY (exchange_id)
        REFERENCES exchanges(id)
        ON DELETE CASCADE
) STRICT;
