<h1 style="text-align: center;">ChibiMOS</h1>

<img style="margin: 0 auto 0 auto;" src="https://raw.githubusercontent.com/Hydrostic/ChibiMOS/refs/heads/main/logo.png" />

An OS was written following the rCore tutorial with extra stability and functionality.

Its name originates from the mascot of Visual Arts/Key's project <b>Rewrite</b> 

## Modifications comparing to rCore
1. Use actively maintained OpenSBI as bootloader
2. Use elf to replace xmas_elf due to latter's incorrect behaviors under some circumstances
3. Eliminate some use of `panic!` by defining error and properly handling them

## Shortages
1. Despite careful checks, potential bugs can still exist

## TODO
1. Implement a heap allocator