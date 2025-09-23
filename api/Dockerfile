FROM rust:1.90-bookworm

RUN cargo install --locked bacon
RUN cargo install --locked sqlx-cli

WORKDIR /api

COPY . ${WORKDIR}

ENTRYPOINT ["./entrypoint.sh"]
CMD ["bacon", "--headless", "run"]
