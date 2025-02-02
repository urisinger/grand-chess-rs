EXE = GrandChess

all:
	RUSTFLAGS="-Ctarget-cpu=native" cargo rustc --release -- --emit link=$(EXE)
