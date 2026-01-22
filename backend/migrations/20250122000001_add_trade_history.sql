-- Add trade_history field to store trading history as JSON
ALTER TABLE users ADD COLUMN trade_history TEXT DEFAULT '[]';
