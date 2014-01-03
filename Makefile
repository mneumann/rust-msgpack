build:
	rustpkg build msgpack
	rustpkg build examples/simple

test:
	rustpkg test msgpack

install:
	rustpkg install examples/simple

clean:
	rm -rf lib bin build .rust
