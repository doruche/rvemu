# Rvemu
Rvemu is a __userland__ RISC-V-64 emulator library written in Rust. Modulized design allows for easy extension and modification.<br/>
### ___Suspension of Works___
# Features
Rvemu uses a modular design, allowing for easy extension and modification. 
<br/>
- __Instruction Set__   To add a new instruction set, simply implement the `Decoder` trait for a `XXXDecoder` (e.g. `Rv64IDecoder`), with a bunch of `Executor`s, which is responsible for executing the instruction.
- __Syscall__   To add a new user lib, you should implement the `SyscallHandler` trait for a `XXXSyscallHandler` (e.g. `GlibcSyscallHandler`). Typically you will need to implement massive syscall functions.
---
With above flexibility, it is quite easy to set up a minimal RISC-V environment to test your own code. For example, you can enable `InsnSet::I` only, and implement a `MinilibSyscallHandler` to provide a minimal set of syscalls, such as `putchar`, `exit`, etc. Then you can run your own RISC-V code in this environment.
# Todo
- Add supports for: Rv64M, Rv64F, Rv64D, Rv64C
- Add supports for Glibc, Newlib.
- Add supports for debugging and gdb stub.
- Add supports for multi-threading.
# Example
```rust
struct MiniSyscall;
impl SyscallHandler for MiniSyscall {
    fn handle(&mut self, state: &mut State, guest: &mut GuestMem) -> Result<()> {
        match state.x[17] {
            SYS_EXIT => Err(Error::Exited(state.x[10] as i64)),
            SYS_PUTCHAR => {
                let c = state.x[10] as u8;
                print!("{}", c as char);
                Ok(())
            }
            _ => Err(Error::SyscallUnimplemented(state.x[17], state.pc))
        }
    }
}

let mut rvemu = Emulator::new()
    .decoder(InsnSet::I)
    .decoder(InsnSet::M)
    .decoder(InsnSet::A)
    .syscall(Box::new(MiniSyscall))
    .stack_size(4096 * 2048)
    .build().unwrap();
rvemu.load_elf("path/to/your/elf/file").unwrap();
let exit_code = rvemu.run().unwrap();
```