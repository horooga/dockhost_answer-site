FROM rust:latest AS build
WORKDIR /build

COPY . .
RUN cargo build

FROM ubuntu
WORKDIR /app

COPY --from=build /build/target/debug/answer-app ./main
COPY --from=build /build/static ./static
COPY --from=build /build/.env .

CMD ["./main"]


