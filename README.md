# Rvemu
Rvemu is a __userland__ RISC-V-64 emulator library written in Rust. Modulized design allows for easy extension and modification.
# Features
Rvemu uses a modular design, allowing for easy extension and modification. 
<br/>
- __Instruction Set__   To add a new instruction set, simply implement the `Decoder` trait for a `XXXDecoder` (e.g. `Rv64IDecoder`), with a bunch of `Executor`s, which is responsible for executing the instruction.
- __Syscall__   To add a new user lib, you should implement the `SyscallHandler` trait for a `XXXSyscallHandler` (e.g. `GlibcSyscallHandler`). Typically you will need to implement massive syscall functions.
With above flexibility, it is quite easy to set up a minimal RISC-V environment to test your own code. For example, you can enable `InsnSet::I` only, and implement a `MinilibSyscallHandler` to provide a minimal set of syscalls, such as `putchar`, `exit`, etc. Then you can run your own RISC-V code in this environment.
# Todo
- Add supports for: Rv64M, Rv64A, Rv64F, Rv64D, Rv64C, Rv64V
- Add supports for Glibc, Newlib.
- Add supports for debugging and gdb stub.
- Add supports for multi-threading.
# Example
```rust
let mut rvemu = Emulator::new()
    .decoder(InsnSet::I)
    .decoder(InsnSet::M)
    .decoder(InsnSet::A)
    .syscall(Box::new(Newlib))
    .stack_size(4096 * 2048)
    .build().unwrap();
let exit_code = rvemu.run().unwrap();
```