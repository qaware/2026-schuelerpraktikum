FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev binaryen nodejs npm
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

