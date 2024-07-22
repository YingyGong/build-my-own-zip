BIN_PATH := bin

all: $(BIN_PATH)/myzip0 $(BIN_PATH)/myunzip0 $(BIN_PATH)/inflate $(BIN_PATH)/huffman $(BIN_PATH)/lz77  $(BIN_PATH)/myunzip $(BIN_PATH)/myzip

huffman: $(BIN_PATH)/huffman

myunzip: $(BIN_PATH)/myunzip

myzip: $(BIN_PATH)/myzip

$(BIN_PATH)/%: 
	@mkdir -p $(BIN_PATH)
	@echo "Building $*"
	@cargo build --release --bin $*
	@cp target/release/$* $(BIN_PATH)/

clean:
	@echo "Cleaning up..."
	@rm -f $(BIN_PATH)/*
	@cargo clean

.PHONY: all clean
