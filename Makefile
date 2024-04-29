EXE = GrandChess

all:
	cargo rustc --release -- --emit link=$(EXE)
