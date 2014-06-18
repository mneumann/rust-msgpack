.PHONY: lib all examples display test clean

RUSTC?=rustc

LIBNAME := $(shell ${RUSTC} --crate-file-name src/msgpack/lib.rs)

all: lib examples test

lib: lib/$(LIBNAME)

lib/$(LIBNAME): src/msgpack/lib.rs src/msgpack/rpc.rs
	@mkdir -p lib
	${RUSTC} -O --out-dir lib $<

test: bin/msgpack-test
	./bin/msgpack-test

bin/msgpack-test: src/msgpack/test.rs lib/$(LIBNAME)
	@mkdir -p bin
	${RUSTC} --test -O -o bin/msgpack-test -L lib $<

examples: bin/simple bin/value

bin/simple: src/examples/simple/main.rs lib/$(LIBNAME)
	@mkdir -p bin
	${RUSTC} -o bin/simple -L lib $<

bin/value: src/examples/value/main.rs lib/$(LIBNAME)
	@mkdir -p bin
	${RUSTC} -o bin/value -L lib $<

clean:
	-$(RM) -r bin
	-$(RM) -r lib

display: bin/value
	ruby -rmsgpack -e "p File.read(ARGV.shift)" test.msgpack
	ruby -rmsgpack -e "p MessagePack.load(File.read(ARGV.shift))" test.msgpack
	./bin/value test.msgpack
