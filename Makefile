build:
	mkdir -p lib bin
	rustc --out-dir lib -O src/msgpack/lib.rs
	rustc -L lib -o bin/simple src/examples/simple/main.rs
	rustc -L lib -o bin/value -O src/examples/value/main.rs

test: build
	rustc -L lib -o bin/test --test src/msgpack/test.rs
	./bin/test

clean:
	rm -rf lib bin

display:
	ruby -rmsgpack -e "p File.read(ARGV.shift)" test.msgpack
	ruby -rmsgpack -e "p MessagePack.load(File.read(ARGV.shift))" test.msgpack
	./bin/value test.msgpack
