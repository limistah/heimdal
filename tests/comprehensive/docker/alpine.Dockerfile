FROM alpine:latest

# Install dependencies
RUN apk add --no-cache \
    git \
    curl \
    bash \
    findutils \
    coreutils

# Create test user
RUN adduser -D -s /bin/bash testuser

# Switch to test user
USER testuser
WORKDIR /home/testuser

# Set up git config
RUN git config --global user.name "Heimdal Test" && \
    git config --global user.email "test@heimdal.test" && \
    git config --global init.defaultBranch main

CMD ["/bin/bash"]
