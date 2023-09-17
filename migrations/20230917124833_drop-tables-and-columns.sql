-- Add migration script here
DROP TRIGGER IF EXISTS update_apps_updated_at ON apps CASCADE;
ALTER TABLE users DROP COLUMN app_id;
DROP TABLE apps;

