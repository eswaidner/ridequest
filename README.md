# RideQuest

Quest-like performance goals and progress visualization for Strava athletes.

### Environment Setup

- .env_secrets
  - DB_PASSWORD (password for the local db)
  - STRAVA_CLIENT_SECRET (strava app secret for auth)

### Usage

Local development is managed with `docker compose`.

Run `make up` to stand up the stack. Frontend is hosted at http://localhost:5173.

Run `make down` to tear down the stack.

See `Makefile` for more utilities.

### Current State

Extremely bare-bones front end GUI supporting Strava authentication flow. Back end facilitates session-based auth.

### Next Steps

Fetch/aggregate activity statistics for a logged in athlete and serve quest/stat progress data to the front end for visualization.
