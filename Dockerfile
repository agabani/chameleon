FROM rust:latest AS build

RUN apt update && \
    apt install -y musl-dev musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    rustup target add wasm32-unknown-unknown && \
    cargo install --locked trunk

WORKDIR /home/appuser/app

COPY . .

RUN SQLX_OFFLINE=true && \
    cargo build --target x86_64-unknown-linux-musl --release
RUN trunk build

FROM scratch AS rootfs

COPY --from=build /home/appuser/app/target/x86_64-unknown-linux-musl/release/chameleon-backend chameleon-backend
COPY --from=build /home/appuser/app/dist dist

FROM scratch

USER 10001:10001

COPY --chown=0:0 --from=rootfs . .

ENV CHAMELEON_LOG="info,sqlx=warn"

CMD [ "/chameleon-backend" ]
