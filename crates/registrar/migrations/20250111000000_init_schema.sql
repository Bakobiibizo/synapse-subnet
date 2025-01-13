-- Add migration script here
CREATE TABLE IF NOT EXISTS subnet_modules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    repo_url TEXT NOT NULL,
    branch TEXT NOT NULL,
    description TEXT,
    author TEXT,
    license TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    downloads INTEGER NOT NULL DEFAULT 0,
    module_type TEXT NOT NULL,
    status TEXT NOT NULL
);
