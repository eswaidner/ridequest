#!/bin/sh

# run database migrations
echo $DATABASE_URL
sqlx migrate run || exit

exec "$@"
