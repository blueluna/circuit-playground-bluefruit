# Rust experiments on the Adafruit Circuit Playground Bluefruit

## The Setup

Following setup was used when running these examples.

The Adafruit Circuit Playground Bluefruit (CPB) is connected to a J-Link EDU
through the SWD interface on the back of the CPB. A USB-to-serial adapter is
connected to the GNG/RX/TX connectors on the edge of the board.

## Debugging

[JLinkGDBServer] from Segger is used to debug, see the `jlinkgdb` shell script
on how JLinkGDBServer is invoked.

Start the GDB server with `jlinkgdb`.

```
$ ./jlinkgdb
```

Then run the program

```
$ cargo run --example neopixel
```

cargo will use the run definition found in `.cargo/config` to launch `gdb` with
the `jlink.gdb` script file.

Issue the gdb command `continue`/`c` to run the program in the debugger.

[JLinkGDBServer]: https://www.segger.com/products/debug-probes/j-link/tools/j-link-gdb-server/about-j-link-gdb-server/
