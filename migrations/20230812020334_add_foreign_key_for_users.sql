-- Add migration script here
ALTER TABLE users ADD CONSTRAINT fk_users_app_id FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE;
