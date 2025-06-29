CREATE TABLE guild_settings (
    guild INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,

    PRIMARY KEY (guild, key)
) STRICT;
