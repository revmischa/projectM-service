# Start from the official Ubuntu 24.04 image
FROM ubuntu:24.04

# Install required packages
RUN apt-get update && \
    apt-get install -y \
        build-essential \
        llvm \
        git \
        cmake \
        ca-certificates \
        libssl-dev \
        curl \
        xvfb \
        libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
        sudo

# Clone the projectM repository and build it
RUN git clone --depth 1 https://github.com/projectM-visualizer/projectm.git /tmp/projectm && \
    cd /tmp/projectm && \
    git fetch --all --tags && \
    git submodule init && \
    git submodule update && \
    mkdir build && \
    cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr/local .. && \
    make -j8 && \
    make install && \
    rm -rf /tmp/projectm

# Get the projectM preset pack
RUN git clone --depth 1 https://github.com/projectM-visualizer/presets-cream-of-the-crop.git /var/task/projectm/presets

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set environment variables for projectM and the GStreamer plugin
ENV PROJECTM_ROOT=/usr/local
ENV GST_PLUGIN_PATH=/var/task/projectm/gstreamer-plugins

# Clone the gst-projectm repository and build the GStreamer plugin
RUN git clone --branch preset-loading https://github.com/projectM-visualizer/gst-projectm.git /tmp/gst-projectm
WORKDIR /tmp/gst-projectm
RUN ./setup.sh --auto
RUN mkdir build && \
    cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release .. && \
    make
RUN ls
RUN pwd
RUN ls
RUN mkdir -p /var/task/projectm/gstreamer-plugins && \
    mv build/libgstprojectm.so $GST_PLUGIN_PATH/ && \
    rm -rf /tmp/gst-projectm
# Create the Rust application
WORKDIR /usr/src/projectm_lambda
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Clean up unnecessary packages to reduce image size
RUN apt-get remove -y \
        build-essential \
        git \
        cmake && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
# remove rust toolchain
RUN rm -rf /root/.cargo

WORKDIR /var/task

# entrypoint shell script is used to start the Xvfb and the Rust application
COPY entrypoint.sh .
RUN chmod +x entrypoint.sh

# set to 4 or 5 for more verbose logging
ENV GST_DEBUG=3
ENV EGL_LOG_LEVEL=debug
ENV RUST_BACKTRACE=1
ENV RUST_LOG=debug

ENV PRESETS_DIR=/var/task/projectm/presets
ENV XDG_RUNTIME_DIR=/tmp
ENV LIBGL_ALWAYS_SOFTWARE=true
ENV MESA_GL_VERSION_OVERRIDE=3.3
ENV EGL_PLATFORM=surfaceless

ENTRYPOINT ["/var/task/entrypoint.sh"]
