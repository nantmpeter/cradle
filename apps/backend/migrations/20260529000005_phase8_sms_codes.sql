-- Add phone field to users table for SMS login
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone VARCHAR(20);

-- SMS verification codes table
CREATE TABLE IF NOT EXISTS sms_verification_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone VARCHAR(20) NOT NULL,
    code VARCHAR(6) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sms_codes_phone ON sms_verification_codes(phone);
CREATE INDEX idx_sms_codes_expires_at ON sms_verification_codes(expires_at);
