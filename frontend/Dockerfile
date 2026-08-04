FROM rust:1-alpine as builder

RUN apk add --no-cache musl-dev binaryen
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk

WORKDIR /app

COPY Cargo.toml Trunk.toml index.html ./
COPY src ./src

RUN trunk build --release

FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
