a.out: main.cpp generated.h src/generated.rs src/lib.rs ../../target/release/libexample_simple_merged.a
	${CXX} -Werror main.cpp generated.cpp -g -L ../../target/release/ -l example_simple_merged

../../target/release/libexample_simple_merged.a: FORCE
	cargo build --release

generated.h generated.cpp ./src/generated.rs: one.zng two.zng
	cd ../../zngur-cli && cargo run gm ../examples/simple_merge/one.zng ../examples/simple_merge/two.zng --in ../examples/simple_merge

FORCE: ;
