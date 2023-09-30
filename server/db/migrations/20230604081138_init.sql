-- migrate:up

CREATE TABLE IF NOT EXISTS apps (
  id uuid PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  type text NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS apps_name_uindex ON apps (name);

-- migrate:down

DROP INDEX IF EXISTS apps_name_uindex;
DROP TABLE IF EXISTS apps;

