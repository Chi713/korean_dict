FROM python:3 as khaiii_builder

RUN apt-get update && apt-get install -y build-essential git curl
RUN git clone https://github.com/Chi713/khaiii.git
WORKDIR /khaiii
RUN pip install cython cmake
RUN pip install --upgrade pip
RUN mkdir build
WORKDIR /khaiii/build
#make and install khaiii
RUN cmake -E env CXXFLAGS="-w" cmake ..
RUN make all
RUN make resource
RUN make install
RUN make package_python
WORKDIR /khaiii/build/package_python/

FROM rust:1.71-bookworm as rust_builder
RUN apt-get update && apt-get install -y build-essential
RUN apt-get install -y python3 python3-pip
ENV SQLX_OFFLINE=true
WORKDIR /korean_dict
COPY .. .
WORKDIR /korean_dict
RUN cargo build -p korean_dict_server --release
RUN rustup target add wasm32-unknown-unknown
WORKDIR /korean_dict/frontend
RUN cargo install trunk
RUN trunk build --public-url /

FROM python:3
RUN apt-get update -y
#RUN apt-get install -y language-pack-ko
#install khaiii
RUN pip install cython cmake
RUN pip install --upgrade pip
ENV CXXFLAGS="-w"
COPY --from=khaiii_builder /khaiii /khaiii
WORKDIR /khaiii/build/package_python/
RUN pip install /khaiii/build/package_python/
#copy over server binary
WORKDIR /korean_dict
COPY --from=rust_builder /korean_dict/target/release/korean_dict_server /korean_dict
RUN mkdir /korean_dict/dist
COPY --from=rust_builder /korean_dict/dist /korean_dict/dist
ENTRYPOINT ["/korean_dict_server","-a","0.0.0.0","--static-dir", "./dist"]
EXPOSE 3000
