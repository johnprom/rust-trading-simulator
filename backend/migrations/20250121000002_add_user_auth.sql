-- Add password field for authentication
ALTER TABLE users ADD COLUMN password_hash TEXT;

-- Create index on username for faster lookups
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
