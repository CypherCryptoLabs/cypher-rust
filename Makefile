include .env
TARGET = cypher-rust

.PHONY: all clean run

all: $(TARGET)

$(TARGET):
	cargo build --release

clean:
	cargo clean

run: $(TARGET)
	env $(shell cat .env | xargs) cargo run