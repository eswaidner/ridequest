FROM rust:1.90-bookworm

RUN cargo install --locked bacon

WORKDIR /api

COPY . ${WORKDIR}

CMD ["bacon", "--headless", "run"]
