# Debian base, not Alpine: Trunk downloads prebuilt glibc-linked
# wasm-bindgen/wasm-opt binaries that cannot run against musl.
FROM rust:1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Bookworm's nodejs is 18. The @tailwindcss/oxide native binding declares
# `engines: node >= 20` and is an *optional* dep, so npm silently skips it on
# 18 and the Tailwind CLI dies with "Cannot find native binding" at build time.
COPY --from=node:22-bookworm-slim /usr/local/bin/node /usr/local/bin/node
COPY --from=node:22-bookworm-slim /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s /usr/local/lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s /usr/local/lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY Cargo.toml Cargo.lock Trunk.toml index.html input.css ./
COPY src ./src
COPY public ./public

RUN trunk build --release

FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]

