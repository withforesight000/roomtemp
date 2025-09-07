CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT,
    encrypted_access_token BLOB NOT NULL,
    encrypted_access_token_nonce BLOB NOT NULL,
    use_proxies BOOLEAN NOT NULL DEFAULT false,
    proxy_url TEXT
);
