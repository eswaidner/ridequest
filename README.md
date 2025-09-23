# RideQuest

Quest-like performance goals and progress visualization for Strava athletes.

### Environment Setup

- Dependencies

  - Docker
  - Rust
  - Node

- .env_secrets
  - DB_PASSWORD (password for the local db)
  - STRAVA_CLIENT_SECRET (strava app secret for auth)

### Usage

Run `make up` to stand up the stack. Frontend is hosted at https://localhost:5173.

Run `make down` to tear down the stack.

See `Makefile` for more utilities.
