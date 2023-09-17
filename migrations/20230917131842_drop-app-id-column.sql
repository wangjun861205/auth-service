-- Add migration script here
ALTER TABLE users DROP COLUMN IF EXISTS app_id;
