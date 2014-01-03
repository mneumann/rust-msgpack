build:
	rustpkg build msgpack
	rustpkg build examples/simple
	rustpkg build examples/parse

test:
	rustpkg test msgpack

install:
	rustpkg install examples/simple
	rustpkg install examples/parse

clean:
	rm -rf lib bin build .rust
