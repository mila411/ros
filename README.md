# ROS (Rust Operating System)

A simple educational operating system for my use that provides basic OS functionality implemented in Rust.

## Functions

### Boot process
- Multi-boot support
- GDT/IDT configuration
- Initialize memory management

### Device management
- Interrupt controller (PIC) configuration
- Keyboard driver
- VGA driver (text mode)

### Memory Management
- Paging implementation
- Heap allocator
- Memory map management

### Shell functions
- Basic command line processing
- Command History
- The following commands are implemented: `help`: display command list
  - `help`: display command list
  - `clear`: clear the screen.
  - `exit`: exit the system.
  - `ls`: display directory contents.
  - `pwd`: display current directory
  - `cd`: Move a directory
  - `mkdir`: Create a directory
  - `touch`: Create a file
  - `time`: Display current time (time zone support)

### File System
- In-memory file system
- Basic file operations
- Directory hierarchy

## What you need

- Rust (nightly)
- QEMU
- cargo-bootimage

## Setup

```bash
## Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install nightly toolchain
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
rustup component add llvm-tools-preview

# Install QEMU
brew install qemu
build

cargo build

## run

``cargo run

### Development status

- [x] Boot process
- [x] Interrupt handling
- [x] Keyboard Input
- [x] Shell Basics
- [x] Basic file system functions
- [x] Time display with time zone support

### Unimplemented

1. Shell Functions
- [] Tab completion
- [] Piping of commands
- [] Input/output redirection
- [] Editing input with cursor keys
- [] Alias setting for commands
2. File system
- [] Reading file contents (cat command)
- [] Deleting files (rm command)
- [] Moving/renaming files (mv command)
- [] Managing file permissions
- [] File system persistence
3.  Process management
- [] Process Creation and Execution
- [] Interprocess Communication
- [] Background Execution
- [] Job Control
4.  Memory management
- [] Swap space management
- [] Memory fragmentation prevention
- [] Virtual memory expansion
5.  Device management
- [] Serial port communication
- [] Mouse drivers
- [] Network functions
- [] Sound functions
6.  System Management
- [] User Management
- [] Log function
- [] Save/Load system settings
- [] Improved shutdown handling
7.  Testing
- [] Expanded unit testing
- [] Additional integration testing
- [] Performance testing
8. Security
- [] Access Control
- [] Memory Protection
- [] System Call Restrictions
