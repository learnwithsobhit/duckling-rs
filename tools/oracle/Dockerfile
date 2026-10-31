# Haskell oracle for the Duckling Rust port: builds the pinned upstream Duckling
# (with an instrumented Duckling.Api overlay) and runs its test suite / exporters.
FROM haskell:9.2.8-buster
RUN sed -i 's|deb.debian.org|archive.debian.org|g; s|security.debian.org|archive.debian.org|g' /etc/apt/sources.list || true
RUN apt-get -o Acquire::Check-Valid-Until=false update -qq && \
    apt-get install -qq -y libpcre3 libpcre3-dev build-essential pkg-config libgmp-dev zlib1g-dev --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*
ENV LANG=C.UTF-8 STACK_ROOT=/stack
WORKDIR /work