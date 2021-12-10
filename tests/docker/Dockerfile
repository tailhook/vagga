FROM busybox

COPY Dockerfile vagga.yaml /will-be-deleted/
COPY Dockerfile vagga.yaml /test/
RUN id
RUN mount
RUN ls -l /
RUN ls -l /will-be-deleted
RUN rm -rf /will-be-deleted
RUN rm -f /test/Dockerfile
