FROM archlinux:latest

# Install dependencies
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm \
    git \
    curl \
    bash \
    findutils \
    which \
    && pacman -Scc --noconfirm

# Create test user
RUN useradd -m -s /bin/bash testuser

# Switch to test user
USER testuser
WORKDIR /home/testuser

# Set up git config
RUN git config --global user.name "Heimdal Test" && \
    git config --global user.email "test@heimdal.test" && \
    git config --global init.defaultBranch main

CMD ["/bin/bash"]
