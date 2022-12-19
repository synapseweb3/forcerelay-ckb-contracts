FROM nervos/ckb-riscv-gnu-toolchain:bionic-20211214

# Set Environment for Rust
ENV RUSTUP_HOME="/opt/local/rustup"
ENV CARGO_HOME="/opt/local/cargo"
ENV PATH="${CARGO_HOME}/bin:${PATH}"
# Install Rust
RUN curl https://sh.rustup.rs -sSf \
    | sh -s -- -y --no-modify-path --profile minimal --default-toolchain nightly-2022-08-01
# Install Rust Components
RUN rustup component add rustfmt
RUN rustup component add clippy
# Install RISC-V target
RUN rustup target add riscv64imac-unknown-none-elf
