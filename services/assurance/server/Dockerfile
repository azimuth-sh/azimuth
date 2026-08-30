ARG AZIMUTH_VERSION=0.1.0-alpha.5

FROM rust:1.87-bookworm AS build
WORKDIR /source

COPY tools/azimuth/Cargo.toml tools/azimuth/Cargo.lock tools/azimuth/
COPY services/assurance/Cargo.toml services/assurance/Cargo.lock services/assurance/
COPY services/assurance/domain/Cargo.toml services/assurance/domain/
COPY services/assurance/server/Cargo.toml services/assurance/server/
# Keep the locked third-party graph in an exported layer that source-only changes do not replace.
RUN mkdir -p tools/azimuth/src services/assurance/domain/src services/assurance/server/src \
    && printf 'pub fn dependency_cache_seed() {}\n' > tools/azimuth/src/lib.rs \
    && printf 'fn main() {}\n' > tools/azimuth/src/main.rs \
    && printf 'pub fn dependency_cache_seed() {}\n' > services/assurance/domain/src/lib.rs \
    && printf 'fn main() {}\n' > services/assurance/server/src/main.rs \
    && cargo build --locked --release --manifest-path services/assurance/Cargo.toml \
        -p azimuth-assurance-server

COPY tools/azimuth tools/azimuth
COPY services/assurance services/assurance
RUN cargo clean --release --manifest-path services/assurance/Cargo.toml \
        -p azimuth -p azimuth-assurance-domain -p azimuth-assurance-server \
    && cargo build --locked --release --manifest-path services/assurance/Cargo.toml \
        -p azimuth-assurance-server

FROM debian:bookworm-slim AS runtime
ARG AZIMUTH_VERSION
LABEL org.opencontainers.image.title="Azimuth Assurance API" \
      org.opencontainers.image.url="https://azimuth.sh" \
      org.opencontainers.image.source="https://github.com/azimuth-sh/azimuth" \
      org.opencontainers.image.version="$AZIMUTH_VERSION" \
      org.opencontainers.image.licenses="Apache-2.0"
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /source/services/assurance/target/release/azimuth-assurance-server /usr/local/bin/azimuth-assurance-server
ENV ASSURANCE_ADDRESS=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["azimuth-assurance-server"]
