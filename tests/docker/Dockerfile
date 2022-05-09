FROM busybox

COPY Dockerfile vagga.yaml /will-be-deleted/
COPY Dockerfile vagga.yaml /test/

# Test whiteouts
RUN rm -rf /will-be-deleted
RUN rm -f /test/Dockerfile

# Override file
RUN echo -n 'Hello' > /hello.txt
RUN echo -n ' world!' >> /hello.txt

# Override symlink
RUN ln -s nonexistent /hi.txt
RUN rm -r /hi.txt && ln -s hello.txt /hi.txt

# Override hardlink
RUN echo -n 'Bye!' > /bye.txt && ln /bye.txt /see-you.txt
RUN echo -n 'Bye-bye!' > /bye-bye.txt && ln -f /bye-bye.txt /see-you.txt

# Override directory
RUN mkdir /empty
RUN rmdir /empty && touch /empty
