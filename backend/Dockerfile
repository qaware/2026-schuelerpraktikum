FROM golang:1.26-alpine AS builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY src ./src
RUN CGO_ENABLED=0 go build -o /app/backend ./src

FROM alpine:latest

WORKDIR /app
COPY --from=builder /app/backend /app/backend

EXPOSE 8585

CMD ["/app/backend"]