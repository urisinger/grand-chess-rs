EXE = GrandChess

all:
	RUSTFLAGS="-Ctarget-cpu=native -C link-args=-Wl,-zstack-size=4194304" cargo rustc --release -- --emit link=$(EXE)
