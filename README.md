GbJit
===

An attempt at an x86\_64 JIT gameboy (and possible gameboy colour) emulator.
The intention is to map gb instructions to some number of x86 instructions,
and then just execute from there, with occasional extra code generation
in the case of self-modifying code.

Compilation Model
---

### Register mapping
- AF maps to al,ah (this one is a bit backwards, because the LAHF puts eflags in ah)
- BC maps to bh,bl
- DE maps to ch,cl
- HL maps to dh,dl
- SP maps to r12w
- PC maps to r13w
- The cycle count is stored in r14
- The cycle count an interrupt will be generated at is in r15
- The address of the memory object is stored in rbp
- The address of a pointer to an interrupt flag is in r11

TODO: Determine if some of those should be on the stack instead of a register

### Execution model
Each 256-byte page of instructions is compiled separately, and has an entry
and exit routine.
