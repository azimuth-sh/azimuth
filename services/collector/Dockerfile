FROM rust:1.87-bookworm AS build
WORKDIR /source
COPY tools/azimuth tools/azimuth
RUN cargo build --locked --release --manifest-path tools/azimuth/Cargo.toml
FROM python:3.13-slim
WORKDIR /app
COPY --from=build /source/tools/azimuth/target/release/azimuth /usr/local/bin/azimuth
COPY packages/python/azimuth_run /app/azimuth_run
COPY services/collector/collector.py services/collector/contexts.py /app/
ENV PYTHONUNBUFFERED=1 PYTHONDONTWRITEBYTECODE=1
USER 65532:65532
ENTRYPOINT ["python", "/app/collector.py"]
