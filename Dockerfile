FROM gcr.io/distroless/cc-debian12

ARG PROFILE=release

COPY ./target/${PROFILE}/rating-exchange-bot /rating-exchange-bot

CMD ["./rating-exchange-bot"]
