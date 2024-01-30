FROM alpine:3.19
WORKDIR /app
RUN apk add --no-cache bash sqlite gzip coreutils
COPY backupdb.sh .
COPY sleeper.sh .
#CMD ["watch", "-n300", "/app/backupdb.sh"] # annoyingly eats stdout
CMD ["/app/sleeper.sh"]
