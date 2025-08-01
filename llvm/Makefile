PREFIX      ?= /usr/local
HELPER_PATH  = $(PREFIX)/lib/afl
BIN_PATH     = $(PREFIX)/bin

VERSION     = $(shell grep '^\#define VERSION ' ./include/config.h | cut -d '"' -f2)

LLVM_CONFIG ?= llvm-config

# CFLAGS      ?= -O3 -funroll-loops
CFLAGS      += -Wall -D_FORTIFY_SOURCE=2 -g -Wno-pointer-sign \
               -DAFL_PATH=\"$(HELPER_PATH)\" -DBIN_PATH=\"$(BIN_PATH)\" \
               -DVERSION=\"$(VERSION)\" 
ifdef AFL_TRACE_PC
  CFLAGS    += -DUSE_TRACE_PC=1
endif

CXXFLAGS    ?= -O3 -funroll-loops
CXXFLAGS    += -Wall -D_FORTIFY_SOURCE=2 -g -Wno-pointer-sign \
               -DVERSION=\"$(VERSION)\" -Wno-variadic-macros

# Mark nodelete to work around unload bug in upstream LLVM 5.0+
CLANG_CFL    = `$(LLVM_CONFIG) --cxxflags` -fno-rtti -fpic $(CXXFLAGS)
CLANG_LFL    = `$(LLVM_CONFIG) --ldflags` $(LDFLAGS)

# User teor2345 reports that this is required to make things work on MacOS X.

ifeq "$(shell uname)" "Darwin"
  CLANG_LFL += -Wl,-flat_namespace -Wl,-undefined,suppress
endif

# We were using llvm-config --bindir to get the location of clang, but
# this seems to be busted on some distros, so using the one in $PATH is
# probably better.

ifeq "$(origin CC)" "default"
  CC         = clang
  CXX        = clang++
endif

ifndef AFL_TRACE_PC
  PROGS      = ./rustc-demangle/target/release/librustc_demangle.a ./afl-clang-fast ./afl-llvm-pass.so ./afl-llvm-rt.o ./afl-llvm-rt-32.o ./afl-llvm-rt-64.o
else
  PROGS      = ./rustc-demangle/target/release/librustc_demangle.a ./afl-clang-fast ./afl-llvm-rt.o ./afl-llvm-rt-32.o ./afl-llvm-rt-64.o
endif

all: test_deps $(PROGS) all_done

test_deps:
ifndef AFL_TRACE_PC
	@echo "[*] Checking for working 'llvm-config'..."
	@which $(LLVM_CONFIG) >/dev/null 2>&1 || ( echo "[-] Oops, can't find 'llvm-config'. Install clang or set \$$LLVM_CONFIG or \$$PATH beforehand."; echo "    (Sometimes, the binary will be named llvm-config-3.5 or something like that.)"; exit 1 )
else
	@echo "[!] Note: using -fsanitize=trace-pc mode (this will fail with older LLVM)."
endif
	@echo "[*] Checking for working '$(CC)'..."
	@which $(CC) >/dev/null 2>&1 || ( echo "[-] Oops, can't find '$(CC)'. Make sure that it's in your \$$PATH (or set \$$CC and \$$CXX)."; exit 1 )
	@echo "[*] Checking for nlohmann/json..."
	@test -f /usr/include/nlohmann/json.hpp || ( echo "[-] Oops, can't find nlohmann/json. Install nlohmann-json3-dev (apt-get install nlohmann-json3-dev)"; exit 1 )
	@echo "[+] All set and ready to build."

./afl-clang-fast: ./src/afl-clang-fast.c | test_deps
	$(CC) $(CFLAGS) $< -o $@ $(LDFLAGS)
	ln -sf afl-clang-fast ./afl-clang-fast++

./rustc-demangle/target/release/librustc_demangle.a: ./rustc-demangle/crates/capi/src/lib.rs ./rustc-demangle/crates/capi/Cargo.toml
	cd rustc-demangle && cargo build -p rustc-demangle-capi --release

./afl-llvm-pass.so: ./src/afl-llvm-pass.so.cc | test_deps
	$(CC) $(CLANG_CFL) -fPIC -c $< -o $@.tmp.o
	ld -r -o $@.o $@.tmp.o ./rustc-demangle/target/release/librustc_demangle.a
	$(CXX) $(CLANG_CFL) -shared $@.o -o $@ $(CLANG_LFL)
	rm -f $@.tmp.o $@.o

./afl-llvm-rt.o: ./src/afl-llvm-rt.o.c | test_deps
	$(CC) $(CFLAGS) -fPIC -c $< -o $@
	ar rcs libafl-llvm-rt.a $@

./afl-llvm-rt-32.o: ./src/afl-llvm-rt.o.c | test_deps
	@printf "[*] Building 32-bit variant of the runtime (-m32)... "
	@$(CC) $(CFLAGS) -m32 -fPIC -c $< -o $@ 2>/dev/null; if [ "$$?" = "0" ]; then echo "success!"; else echo "failed (that's fine)"; fi

./afl-llvm-rt-64.o: ./src/afl-llvm-rt.o.c | test_deps
	@printf "[*] Building 64-bit variant of the runtime (-m64)... "
	@$(CC) $(CFLAGS) -m64 -fPIC -c $< -o $@ 2>/dev/null; if [ "$$?" = "0" ]; then echo "success!"; else echo "failed (that's fine)"; fi

test_build: $(PROGS)
	@echo "[*] Testing the CC wrapper and instrumentation output..."
	unset AFL_USE_ASAN AFL_USE_MSAN AFL_INST_RATIO; AFL_QUIET=1 AFL_PATH=. AFL_CC=$(CC) ./afl-clang-fast $(CFLAGS) ./test/test-instr.c -o test-instr $(LDFLAGS)
# Use /dev/null to avoid problems with optimization messing up expected
# branches. See https://github.com/google/AFL/issues/30.
	@echo "[+] All right, the instrumentation seems to be working!"

all_done:
	@echo "[+] All done! You can now use './afl-clang-fast' to compile programs."

.NOTPARALLEL: clean

clean:
	rm -f *.o *.so *.a *~ a.out core core.[1-9][0-9]* test-instr .test-instr0 .test-instr1 *.txt
	rm -f $(PROGS) ./afl-clang-fast++
	rm -rf ./rustc-demangle/target