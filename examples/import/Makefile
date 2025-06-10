.PHONY: ../../target/release/libdependency_import.a clean

a.out: main.cpp deps/foo/foo.cpp deps/bar/bar.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libdependency_import.a
	${CXX} -std=c++11 main.cpp deps/foo/foo.cpp deps/bar/bar.cpp -I. -g -L ../../target/release/ -l dependency_import

../../target/release/libdependency_import.a:
	cargo build --release

generated.h ./src/generated.rs: main.zng
	cd ../../zngur-cli && cargo run g ../examples/import/main.zng --cargo-manifest ../examples/import/Cargo.toml

clean:
	rm -f a.out generated.h src/generated.rs
	rm -rf a.out.dSYM
	-cargo clean
