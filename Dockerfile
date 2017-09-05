FROM rust

WORKDIR /source
COPY . /source
RUN cargo build --release

FROM debian

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get -y --no-install-recommends install \
      ca-certificates \
      libssl-dev && \
    rm -rf /var/lib/apt/lists/*
COPY --from=0 /source/target/release/ecr-cleaner /usr/local/bin/ecr-cleaner
ENTRYPOINT ["/usr/local/bin/ecr-cleaner"]
