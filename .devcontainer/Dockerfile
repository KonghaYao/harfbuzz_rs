FROM konghayao123/best-cn-dev-container:1.1.0

RUN . /init/rust.sh
RUN . "$HOME/.cargo/env"
RUN apt install llvm -y
RUN apt install libclang-dev -y
RUN cargo install bindgen-cli
