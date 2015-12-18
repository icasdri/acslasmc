# acslasmc – ACSL Assembly to C compiler
**acslasmc** is a direct-translation cross-compiler that compiles code written in the fictional ACSL Assembly Language to ANSI C which can then be legitimately compiled to runnable code by a C compiler such as *gcc* or *clang*.

## Usage

    acslasmc SOURCE [OUTPUT]

Compiles ACSL Assembly code found in the file SOURCE and places the resultant C code into OUTPUT. If OUTPUT is not specified, it defaults to "SOURCE.out.c"

## Building
**acslasmc** can be built like any other [Cargo](http://doc.crates.io/) crate.

For debug builds, simply run:

    cargo build

And for release builds:

    cargo build --release

## ACSL Assembly Language
The ACSL Assembly Language is a fictional assembly language used in the "short problems" section of the [American Computer Science League](http://acsl.org/) series of contests. Below is a condensed description of the language.

The ACSL Assembly Language runs on a fictional ACSL computer that has unlimited memory. Each memory location described by an alphanumeric address contains an integer in the range -999999 to +999999. A special accumulator (ACC) location in memory stores the result of certain operations.

Each line contains one instruction. Execution starts from the first line and proceeds sequentially except for "branch" instructions. Execution ends either with the `END` instruction or when EOF is encountered. Each line is structured as: 

    LABEL OPCODE LOC *comments*

#### Opcodes
* Some opcodes may take an integer literal of the form `=<integer>` (e.g. `=22`) instead of a memory location. The places where this may occur are denoted with */literal*.
* *Emphasized* symbols stand for "contents of ...".
* Fields not mentioned in an opcode's description are optional. (e.g. LABEL is optional almost all the time)

`LOAD`: place *LOC/literal* into ACC
<br>`STORE`: place *ACC* into LOC
<br>`ADD`: add *LOC/literal* to *ACC* (modulo 1000000) and place result in ACC
<br>`SUB`: subtract *LOC/literal* from *ACC* (modulo 1000000) and place result in ACC
<br>`MULT`: multiply *LOC/literal* by *ACC* (modulo 1000000) and place result in ACC
<br>`DIV`: divide *LOC/literal* into *ACC* (modulo 1000000) and place result in ACC
<br>`BE`: branch to instruction labeled *LOC* if *ACC* is 0
<br>`BG`: branch to instruction labeled *LOC* if *ACC* is greater than 0
<br>`BL`: branch to instruction labeled *LOC* if *ACC* is less than 0
<br>`BU`: branch to instruction labeled *LOC* (unconditionally)
<br>`END`: terminate execution
<br>`PRINT`: print *LOC/literal*
<br>`DC`: place contents of LOC taken literally (no `=`sign needed here) into memory location described by LABEL taken literally
