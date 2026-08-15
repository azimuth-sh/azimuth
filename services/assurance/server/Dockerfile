ARG AZIMUTH_VERSION=0.1.0-alpha.1

FROM rust:1.87-bookworm AS build
WORKDIR /source
COPY tools/azimuth tools/azimuth
COPY services/assurance services/assurance
RUN cargo build --release --manifest-path services/assurance/Cargo.toml -p azimuth-assurance-server

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
