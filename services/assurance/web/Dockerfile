ARG AZIMUTH_VERSION=0.1.0-alpha.6

FROM node:22-bookworm-slim AS dependencies
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --ignore-scripts

FROM node:22-bookworm-slim AS build
WORKDIR /app
COPY --from=dependencies /app/node_modules ./node_modules
COPY . .
RUN npm run build

FROM node:22-bookworm-slim AS runtime
ARG AZIMUTH_VERSION
LABEL org.opencontainers.image.title="Azimuth Assurance Web" \
      org.opencontainers.image.url="https://azimuth.sh" \
      org.opencontainers.image.source="https://github.com/azimuth-sh/azimuth" \
      org.opencontainers.image.version="$AZIMUTH_VERSION" \
      org.opencontainers.image.licenses="Apache-2.0"
WORKDIR /app
ENV NODE_ENV=production
ENV PORT=3000
COPY --from=build /app/.next/standalone ./
COPY --from=build /app/.next/static ./.next/static
EXPOSE 3000
CMD ["node", "server.js"]
