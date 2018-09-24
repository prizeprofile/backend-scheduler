FROM rust:1.29.0

WORKDIR /home/ec2-user/run
COPY . .

RUN cargo install

EXPOSE 80

CMD ["cargo", "run", "--release"]

