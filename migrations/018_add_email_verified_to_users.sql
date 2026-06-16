-- Add email verification columns to users table
-- Part of v2 Phase 1.5: User table extension

ALTER TABLE users
  ADD COLUMN email_verified     BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN email_verified_at  TIMESTAMPTZ;
