# print demangled symbols by default
set print asm-demangle on

# Connect to the JLink GDB server
target remote :2331

monitor semihosting enable
monitor semihosting IOClient 3

# reset to start
monitor reset

# Load the program
load

stepi