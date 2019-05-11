# Apple1 emulator

The CPU emulator is in the separate repository:

* [CPU MOS 6502 emulator](https://github.com/alexander-akhmetov/mos6502).

![apple1](https://github.com/alexander-akhmetov/apple1/blob/master/image.jpg?raw=true)


## Minichess

```
NEW GAME

!--!--!--!--!--!--!--!--!
!WR!WN!WB!WK!WQ!WB!WN!WR! 0
!--!--!--!--!--!--!--!--!
!WP!WP!WP!**!WP!WP!WP!WP! 1
!--!--!--!--!--!--!--!--!
!**!  !**!  !**!  !**!  ! 2
!--!--!--!--!--!--!--!--!
!  !**!  !WP!  !**!  !**! 3
!--!--!--!--!--!--!--!--!
!**!  !**!  !**!  !**!  ! 4
!--!--!--!--!--!--!--!--!
!  !**!  !**!  !**!  !**! 5
!--!--!--!--!--!--!--!--!
!BP!BP!BP!BP!BP!BP!BP!BP! 6
!--!--!--!--!--!--!--!--!
!BR!BN!BB!BK!BQ!BB!BN!BR! 7
!--!--!--!--!--!--!--!--!

 0  1  2  3  4  5  6  7

WP 13 33
```

## How to use

Run binary:

```
cargo run
```

The command above starts the Apple-1 emulator with Woz Monitor at the address `0xFF00`. You should see the screen and the command line prompt:

```
\
<cursor>
```

With optional flag `-p` you can load an additional program to the memory:

```
cargo run -- -p asm/apple1hello.asm
```

It will be loaded to the memory with starting address `0x7000`. To run it using Woz Monitor type `7000R` and press enter.

You should see this:

```
^?\
7000R

7000: A9
HELLO WORLD!

█
```

To see the hex content of the program: `7000.<END ADDR>`, for example: `7000.700F`:

```
7000.700F

7000: A9 8D 20 EF FF A9 C8 20
7008: EF FF A9 C5 20 EF FF A9
```

## Basic

You can type `E000R` to start basic (run program at `E000`).

Simple BASIC program to try:

```basic
10 FOR I = 1 TO 5
20 PRINT "HELLO, WORLD!"
30 NEXT I
40 END

RUN
```

## Debug

You can disable the screen (`-s`) and enable debug logging:

```
RUST_LOG=debug cargo run -- -s -p asm/apple1hello.asm
```

## Apple 1 Basic

* [BASIC source code listing](https://github.com/jefftranter/6502/blob/master/asm/a1basic/a1basic.s)
* [Disassembled BASIC](http://www.brouhaha.com/~eric/retrocomputing/apple/apple1/basic/)


There are two different ROMs, one of them is from Replica1 (`sys/replica1.bin`) and another one, `roms/apple1basic.bin`. Seems like it has `0xD0F2` instead of `0xD012`.
Both seems to be working well, though I did not test everything.

You can inspect them if you load them to memory and print hex data at location `E3D5.E3DF` with Woz Monitor.

	note: http://www.brielcomputers.com/phpBB3/viewtopic.php?f=10&t=404
	discussion about the same problem

apple1basic.bin:

```
E3D5.E3DF

E3D5: 2C F2 D0
...
```

Replica1 basic content:

```
E3D5.E3DF

E3D5: 2C 12 D0
...
```

### Start Basic

```
cargo run
```

and then type `E000R`.


## Resources

* [6502 instruction set](https://www.masswerk.at/6502/6502_instruction_set.html#BIT)
* [Apple1 BASIC manual](https://archive.org/stream/apple1_basic_manual/apple1_basic_manual_djvu.txt)
* [www.applefritter.com](https://www.applefritter.com)
* [6502 memory test](http://www.willegal.net/appleii/6502mem.htm)
* [apple1 programs](http://hoop-la.ca/apple2/2008/retrochallenge.net.html)
* [apple1 programs 2](http://www.willegal.net/appleii/apple1-software.htm)
* [Woz Monitor description](https://www.sbprojects.net/projects/apple1/wozmon.php)
* [6502 instructions description with undocumented commands](http://www.zimmers.net/anonftp/pub/cbm/documents/chipdata/64doc)
