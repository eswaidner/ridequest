.PHONY:

COMPOSE_CONFIG := --env-file .env_local --env-file .env_secrets

build:
	docker compose ${COMPOSE_CONFIG} build

up: build
	docker compose ${COMPOSE_CONFIG} up -d

down:
	docker compose ${COMPOSE_CONFIG} down

log-app:
	docker logs ridequest_app

log-api:
	docker logs ridequest_api

psql:
	docker compose exec db psql -U ridequest -h localhost -d ridequest