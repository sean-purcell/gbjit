GbJit
===

An attempt at an x86\_64 JIT gameboy (and possible gameboy colour) emulator.
The intention is to map gb instructions to some number of x86 instructions,
and then just execute from there, with occasional extra code generation
in the case of self-modifying code.
