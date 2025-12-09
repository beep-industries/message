-- Add down migration script here

-- Drop the trigger first
DROP TRIGGER IF EXISTS update_messages_updated_at ON messages;

-- Drop the table
DROP TABLE IF EXISTS messages;

-- Note: Not dropping the function update_updated_at_column() as it might be used by other tables
