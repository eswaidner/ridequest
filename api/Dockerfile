FROM rust:1.90-bookworm

RUN cargo install --locked bacon
RUN cargo install --locked sqlx-cli

# TODO run db migrations

WORKDIR /api

COPY . ${WORKDIR}

CMD ["bacon", "--headless", "run"]
