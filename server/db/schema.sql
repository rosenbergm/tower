CREATE TABLE IF NOT EXISTS "schema_migrations" (version varchar(128) primary key);
CREATE TABLE apps (
  id uuid PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  type text NOT NULL
);
CREATE UNIQUE INDEX apps_name_uindex ON apps (name);
-- Dbmate schema migrations
INSERT INTO "schema_migrations" (version) VALUES
  ('20230604081138');
