FROM busybox

COPY Dockerfile vagga.yaml /will-be-deleted/
COPY Dockerfile vagga.yaml /test/
RUN rm -rf /will-be-deleted
RUN rm -f /test/Dockerfile
