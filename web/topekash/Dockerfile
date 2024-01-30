FROM alpine:3.19
WORKDIR /app
RUN apk add --no-cache jq curl units findutils bash
COPY topeka.sh .
COPY sleeper.sh .
#CMD ["watch", "-n120", "/app/topeka.sh"] # annoyingly eats stdout
CMD ["/app/sleeper.sh"]
