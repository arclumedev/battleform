# Auth Specification

## Purpose

Authenticate users via email/password or OAuth (Google, GitHub). Session-based auth persisted in Redis. Supports account linking when the same email appears across providers.

## Requirements

### Requirement: Email registration

The system SHALL allow users to register with an email and password.

#### Scenario: Successful registration

- GIVEN a new email address
- WHEN the user submits email, password (8+ chars), and optional name
- THEN create a User and AuthIdentity with provider "email"
- AND hash the password
- AND start a session
- AND return the user profile

#### Scenario: Duplicate email

- GIVEN an email already registered with provider "email"
- WHEN a new registration is attempted with the same email
- THEN reject with 409 Conflict

#### Scenario: Short password

- GIVEN a password shorter than 8 characters
- WHEN registration is attempted
- THEN reject with 400 Bad Request

### Requirement: Email login

The system SHALL authenticate users with email and password.

#### Scenario: Valid credentials

- GIVEN a registered email and correct password
- WHEN the user submits login
- THEN verify the password hash
- AND start a session
- AND return the user profile

#### Scenario: Invalid credentials

- GIVEN a wrong password
- WHEN login is attempted
- THEN reject with 401 Unauthorized

### Requirement: OAuth login

The system SHALL support Google and GitHub OAuth flows.

#### Scenario: New OAuth user

- GIVEN a user authenticating via Google/GitHub for the first time
- WHEN the OAuth callback fires
- THEN create a User and AuthIdentity with the provider's data
- AND start a session
- AND redirect to the frontend lobby

#### Scenario: Existing OAuth identity

- GIVEN a user who has previously logged in with this provider
- WHEN they authenticate again
- THEN match by provider + provider_subject
- AND update last_used_at
- AND start a session

#### Scenario: Email match across providers

- GIVEN an email verified via one provider
- WHEN the same email appears from a different provider
- THEN link the new AuthIdentity to the existing User

### Requirement: Session management

The system SHALL maintain sessions via Redis-backed cookies.

#### Scenario: Profile access

- GIVEN an authenticated session
- WHEN GET /api/auth/profile is called
- THEN return the user's id, fullName, avatarUrl, systemRole

#### Scenario: Unauthenticated access

- GIVEN no session
- WHEN GET /api/auth/profile is called
- THEN return 401

#### Scenario: Logout

- GIVEN an authenticated session
- WHEN POST /api/auth/logout is called
- THEN destroy the session
