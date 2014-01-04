build:
	rustpkg build msgpack
	rustpkg build examples/simple
	rustpkg build examples/value

test: build
	rustpkg test msgpack

install:
	rustpkg install examples/simple
	rustpkg install examples/value

clean:
	rm -rf lib bin build .rust

display:
	ruby -rmsgpack -e "p File.read(ARGV.shift)" test.msgpack
	ruby -rmsgpack -e "p MessagePack.load(File.read(ARGV.shift))" test.msgpack
	./bin/value test.msgpack
