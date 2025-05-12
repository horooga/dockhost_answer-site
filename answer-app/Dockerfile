FROM rust:latest AS build
WORKDIR /build

COPY . .
RUN --mount=type=cache,dst=target/debug/deps \
    --mount=type=cache,dst=target/debug/build \
    cargo build

FROM ubuntu
WORKDIR /app

COPY --from=build /build/target/debug/answer-app ./main
COPY --from=build /build/static ./static
COPY --from=build /build/.env .
COPY --from=build /build/questions.yaml .

CMD ["./main"]


