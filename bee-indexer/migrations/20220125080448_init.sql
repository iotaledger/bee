--We store addresses as hex strings.

CREATE TABLE IF NOT EXISTS alias_outputs (
    alias_id         VARCHAR PRIMARY KEY NOT NULL,
    output_id        VARCHAR UNIQUE NOT NULL,
    amount           BIGINT NOT NULL,
    state_controller VARCHAR NOT NULL,
    governor         VARCHAR NOT NULL,
    issuer           VARCHAR,
    sender           VARCHAR,
    milestone_index  INT NOT NULL
);

CREATE INDEX alias_state_controller ON alias_outputs ( state_controller );
CREATE INDEX alias_governor ON alias_outputs ( governor );
CREATE INDEX alias_issuer ON alias_outputs ( issuer );
CREATE INDEX alias_sender ON alias_outputs ( sender );

CREATE TABLE IF NOT EXISTS extended_outputs (
    output_id       VARCHAR PRIMARY KEY NOT NULL,
    amount          BIGINT NOT NULL,
    sender          VARCHAR,
    tag             VARCHAR,
    address         VARCHAR NOT NULL,
    milestone_index INT NOT NULL
);

CREATE INDEX extended_sender_tag ON extended_outputs ( sender, tag );
CREATE INDEX extended_address ON extended_outputs ( address );

CREATE TABLE IF NOT EXISTS foundry_outputs (
    foundry_id      VARCHAR PRIMARY KEY NOT NULL,
    output_id       VARCHAR UNIQUE NOT NULL,
    amount          BIGINT NOT NULL,
    sender          VARCHAR,
    tag             VARCHAR,
    address         VARCHAR NOT NULL,
    milestone_index INT NOT NULL
);

CREATE INDEX foundry_address ON foundry_outputs ( address );

CREATE TABLE IF NOT EXISTS nft_outputs (
    nft_id          VARCHAR PRIMARY KEY NOT NULL,
    output_id       VARCHAR UNIQUE NOT NULL,
    amount          BIGINT NOT NULL,
    issuer          VARCHAR,
    sender          VARCHAR,
    tag             VARCHAR,
    address         VARCHAR NOT NULL,
    milestone_index INT NOT NULL
);

CREATE INDEX nft_issuer ON nft_outputs ( issuer );
CREATE INDEX nft_sender_tag ON nft_outputs ( sender, tag );
CREATE INDEX nft_address ON nft_outputs ( address );