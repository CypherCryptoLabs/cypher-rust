include .env
TARGET = cypher-rust

.PHONY: all clean run rm-container-% run-container-% run-local-testnet

all: $(TARGET)

$(TARGET):
	cargo build --release

clean:
	cargo clean

run: $(TARGET)
	env $(shell cat .env | xargs) cargo run

docker:
	docker build -t $(TARGET) .

run-container-%:
	if ! [ -d ./testnet_volumes/$(TARGET)-$* ]; then mkdir ./testnet_volumes/$(TARGET)-$*; fi

	$(eval SEED_IP := $(shell echo $* - 1 | bc))
	docker run -v $(shell pwd)/testnet_volumes/$(TARGET)-$*:/data -d --name $(TARGET)-$* --ip 172.0.0.$* -e CYPHER_EXTERNAL_IP=172.0.0.$* -e CYPHER_SEED_IP=172.0.0.1 -e CYPHER_SEED_WALLET_ADDRESS=0x4010286387220381004084763062714255304864 $(TARGET)

rm-container-%:
	-docker container rm $(TARGET)-$*

stop-container-%:
	-docker container stop $(TARGET)-$*

stop-testnet:
	$(MAKE) stop-container-1
	$(MAKE) stop-container-2
	$(MAKE) stop-container-3
	$(MAKE) stop-container-4

testnet: 
	if ! [ -d "./testnet_volumes" ]; then mkdir "./testnet_volumes"; fi

	$(MAKE) rm-container-1
	$(MAKE) rm-container-2
	$(MAKE) rm-container-3
	$(MAKE) rm-container-4
	$(MAKE) run-container-1
	$(MAKE) run-container-2
	$(MAKE) run-container-3
	$(MAKE) run-container-4
