FROM rust:1.87-bookworm AS build
WORKDIR /source
COPY tools/azimuth tools/azimuth
COPY services/assurance services/assurance
RUN cargo build --release --manifest-path services/assurance/Cargo.toml -p azimuth-assurance-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /source/services/assurance/target/release/azimuth-assurance-server /usr/local/bin/azimuth-assurance-server
ENV ASSURANCE_ADDRESS=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["azimuth-assurance-server"]
