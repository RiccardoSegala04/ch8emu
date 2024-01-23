# ch8emu
ch8emu is a simple Chip-8 emulator written in Rust. This emulator allows you to run programs written for the Chip-8 virtual machine, a popular platform for retro gaming enthusiasts and hobbyists.
<img width="904" alt="Screenshot 2024-01-23 alle 18 49 45" src="https://github.com/RiccardoSegala04/ch8emu/assets/72670063/17e321b0-e5fa-46ff-991c-d9cc71928a59">

## Getting Started
1. Ensure you have Rust installed. If not, follow the instructions at [rustup.rs](rustup.rs) to install it.

2. Clone the repository:
   ```bash
    git clone https://github.com/RiccardoSegala04/ch8emu
    cd ch8emu
   ```
   
3. Build and run the emulator:
   ```bash
   cargo run
   ```
   This will run the default IBM logo test program
   
4. Load a Chip-8 ROM by providing the file path as a command-line argument:
    ```bash
    cargo run roms/your-rom.ch8
    ```

## Controls
The Chip-8 keypad is emulated using your computer's keyboard. The default key mapping is as follows:

```
  1 2 3 C   =>   1 2 3 4
  4 5 6 D        Q W E R
  7 8 9 E        A S D F
  A 0 B F        Z X C V
```

## Contribute
Feel free to contribute to this project! Whether you want to add features, fix bugs, or improve documentation, your contributions are welcome.

