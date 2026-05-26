-- Phase 7: Add avatar_url to users
ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url TEXT;

-- Add avatar_url to MeResponse / UserResponse
-- (No extra tables needed — we store the URL directly on users)
