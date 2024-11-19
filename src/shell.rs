use crate::filesystem;
use crate::{print, println};
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use pc_keyboard::{DecodedKey, KeyCode};

pub struct Shell {
    input_buffer: String,
    cursor_position: usize,
    insert_mode: bool,
    command_history: Vec<String>,
    history_index: usize,
    timezone_offset: i8, // 追加
}

impl Shell {
    pub fn new() -> Shell {
        Shell {
            input_buffer: String::new(),
            cursor_position: 0,
            insert_mode: true,
            command_history: Vec::new(),
            history_index: 0,
            timezone_offset: 9,
        }
    }

    pub fn handle_key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::Unicode('\n') => {
                println!();
                self.execute_command();
            }
            DecodedKey::Unicode(c) => {
                self.input_buffer.insert(self.cursor_position, c);
                self.cursor_position += 1;
                print!("{}", c);
            }
            DecodedKey::RawKey(key) => match key {
                KeyCode::Backspace => self.handle_backspace(),
                KeyCode::Delete => self.handle_delete(),
                KeyCode::Home => self.handle_home(),
                KeyCode::End => self.handle_end(),
                KeyCode::Insert => self.handle_insert(),
                KeyCode::ArrowUp => self.history_up(),
                KeyCode::ArrowDown => self.history_down(),
                _ => {}
            },
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
            self.redraw_line();
        }
    }

    pub fn handle_delete(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
            self.redraw_line();
        }
    }

    pub fn handle_home(&mut self) {
        self.cursor_position = 0;
        self.redraw_line();
    }

    pub fn handle_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
        self.redraw_line();
    }

    pub fn handle_insert(&mut self) {
        self.insert_mode = !self.insert_mode;
    }

    fn redraw_line(&self) {
        print!("\r$ {}", self.input_buffer);
        for _ in self.cursor_position..self.input_buffer.len() {
            print!("\x08");
        }
    }

    fn execute_command(&mut self) {
        println!();

        if !self.input_buffer.is_empty() {
            let parts: Vec<&str> = self.input_buffer.trim().split_whitespace().collect();

            if !parts.is_empty() {
                match parts[0] {
                    "help" => self.cmd_help(),
                    "clear" => self.cmd_clear(),
                    "history" => self.cmd_history(),
                    "exit" => {
                        self.cmd_exit();
                    }
                    "ls" => print!("{}", self.cmd_ls()),
                    "echo" => {
                        if parts.len() > 1 {
                            print!("{}", self.cmd_echo(&parts[1..]));
                        }
                    }
                    "pwd" => print!("{}", self.current_dir_str()),
                    "mkdir" => {
                        if parts.len() > 1 {
                            if let Err(e) = filesystem::create_directory(parts[1]) {
                                println!("mkdir: {}", e);
                            }
                        } else {
                            println!("Usage: mkdir <directory>");
                        }
                    }
                    "cd" => {
                        if parts.len() > 1 {
                            if let Err(e) = filesystem::change_directory(parts[1]) {
                                println!("cd: {}", e);
                            }
                        } else {
                            if let Err(e) = filesystem::change_directory("/") {
                                println!("cd: {}", e);
                            }
                        }
                    }
                    "touch" => {
                        if parts.len() > 1 {
                            self.cmd_touch(parts[1]);
                        } else {
                            println!("Usage: touch <filename>");
                        }
                    }
                    command => println!("Unknown command: '{}'", command),
                }

                self.command_history.push(self.input_buffer.clone());
            }
        }

        self.input_buffer.clear();
        self.cursor_position = 0;
        print!("$ ");
    }

    fn parse_redirects<'a>(&self, parts: &[&'a str]) -> (Vec<&'a str>, Option<(&'a str, &'a str)>) {
        let mut command = Vec::new();
        let mut redirect = None;

        let mut i = 0;
        while i < parts.len() {
            if parts[i] == ">" || parts[i] == ">>" {
                if i + 1 < parts.len() {
                    redirect = Some((parts[i], parts[i + 1]));
                    break;
                }
            } else {
                command.push(parts[i]);
            }
            i += 1;
        }

        (command, redirect)
    }

    fn write_to_file(&self, filename: &str, content: &str, append: bool) {
        match filesystem::write_file(filename, content.as_bytes(), append) {
            Ok(_) => (),
            Err(e) => println!("Error writing to file: {}", e),
        }
    }

    fn cmd_help(&self) {
        println!("Available commands:");
        println!("  help     - Show this help");
        println!("  clear    - Clear screen");
        println!("  history  - Show command history");
        println!("  exit     - Shutdown the system");
        println!("  ls       - List directory contents");
        println!("  echo     - Display a line of text");
        println!("  pwd      - Print working directory");
    }

    fn cmd_clear(&mut self) {
        if let Some(mut writer) = crate::vga_buffer::WRITER.try_lock() {
            writer.clear_screen();
        }
        print!("$ ");
    }

    fn cmd_history(&self) {
        for (i, cmd) in self.command_history.iter().enumerate() {
            println!("{}: {}", i, cmd);
        }
    }

    fn cmd_exit(&self) {
        println!("Shutting down...");
        unsafe {
            let mut port = x86_64::instructions::port::Port::new(0x604);
            port.write(0x2000 as u16); // APMシャットダウン

            let mut qemu_exit_port = x86_64::instructions::port::Port::new(0xf4);
            qemu_exit_port.write(0x10 as u32);

            x86_64::instructions::interrupts::disable();

            loop {
                x86_64::instructions::hlt();
            }
        }
    }

    fn cmd_ls(&self) -> String {
        let mut output = String::new();
        let entries = filesystem::list_current_directory();
        for (name, is_dir) in entries {
            if is_dir {
                output.push_str(&format!("{}/\n", name));
            } else {
                output.push_str(&format!("{}\n", name));
            }
        }
        output
    }

    fn cmd_mkdir(&self, dir_name: &str) {
        match filesystem::create_directory(dir_name) {
            Ok(_) => println!("Directory created: {}", dir_name),
            Err(e) => println!("mkdir: {}", e),
        }
    }

    fn cmd_touch(&self, file_name: &str) {
        match filesystem::create_file(file_name, None) {
            Ok(_) => println!("File created: {}", file_name),
            Err(e) => println!("touch: {}", e),
        }
    }

    fn cmd_cd(&mut self, dir_name: &str) {
        if let Err(e) = filesystem::change_directory(dir_name) {
            println!("cd: {}", e);
        }
    }

    fn cmd_time(&self) {
        let mut rtc_port_cmd = x86_64::instructions::port::Port::<u8>::new(0x70);
        let mut rtc_port_data = x86_64::instructions::port::Port::<u8>::new(0x71);

        unsafe {
            rtc_port_cmd.write(0x04);
            let mut hours = rtc_port_data.read();
            rtc_port_cmd.write(0x02);
            let minutes = rtc_port_data.read();
            rtc_port_cmd.write(0x00);
            let seconds = rtc_port_data.read();

            hours = ((hours >> 4) * 10 + (hours & 0xf)) % 24;
            let minutes = ((minutes >> 4) * 10 + (minutes & 0xf)) % 60;
            let seconds = ((seconds >> 4) * 10 + (seconds & 0xf)) % 60;

            hours = ((hours as i16 + self.timezone_offset as i16) % 24) as u8;

            println!(
                "Current time (UTC{:+}): {:02}:{:02}:{:02}",
                self.timezone_offset, hours, minutes, seconds
            );
        }
    }

    fn cmd_pwd(&self) {
        print!("{}", self.current_dir_str());
    }

    pub fn history_up(&mut self) {
        if !self.command_history.is_empty() && self.history_index < self.command_history.len() {
            self.history_index += 1;
            let index = self.command_history.len() - self.history_index;
            self.input_buffer = self.command_history[index].clone();
            self.cursor_position = self.input_buffer.len();
            self.redraw_line();
        }
    }

    pub fn history_down(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if self.history_index == 0 {
                self.input_buffer.clear();
            } else {
                let index = self.command_history.len() - self.history_index;
                self.input_buffer = self.command_history[index].clone();
            }
            self.cursor_position = self.input_buffer.len();
            self.redraw_line();
        }
    }

    pub fn handle_tab(&mut self) {
        let input = self.input_buffer[..self.cursor_position].trim();

        if input.is_empty() {
            println!("\nAvailable commands:");
            self.cmd_help();
            print!("$ ");
            return;
        }

        let candidates = self.get_completion_candidates(input);

        match candidates.len() {
            0 => (),
            1 => {
                self.input_buffer = candidates[0].clone();
                self.cursor_position = self.input_buffer.len();
                self.redraw_line();
            }
            _ => {
                println!("\nPossible completions:");
                for candidate in candidates {
                    println!("{}", candidate);
                }
                print!("$ {}", self.input_buffer);
            }
        }
    }

    fn get_completion_candidates(&self, input: &str) -> Vec<String> {
        let mut candidates = Vec::new();

        let commands = [
            "help", "clear", "ls", "cd", "pwd", "time", "mkdir", "touch", "exit",
        ];
        for &cmd in commands.iter() {
            if cmd.starts_with(input) {
                candidates.push(String::from(cmd));
            }
        }

        if input.contains(' ') {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if ["cd", "ls", "touch", "mkdir"].contains(&parts[0]) {
                if let Some(prefix) = parts.get(1) {
                    let files = filesystem::list_current_directory();
                    for (name, _) in files {
                        if name.starts_with(prefix) {
                            candidates.push(format!("{} {}", parts[0], name));
                        }
                    }
                }
            }
        }

        candidates
    }

    fn cmd_echo(&self, args: &[&str]) -> String {
        format!("{}\n", args.join(" "))
    }

    fn cmd_help_str(&self) -> String {
        let mut output = String::from("Available commands:\n");
        output.push_str("  help     - Show this help\n");
        output.push_str("  clear    - Clear screen\n");
        output.push_str("  history  - Show command history\n");
        output.push_str("  exit     - Shutdown the system\n");
        output.push_str("  ls       - List directory contents\n");
        output.push_str("  echo     - Display a line of text\n");
        output.push_str("  pwd      - Print working directory\n");
        output
    }

    fn current_dir_str(&self) -> String {
        let current_path = filesystem::get_current_path();
        if current_path.is_empty() {
            "/\n".to_string()
        } else {
            format!("/{}\n", current_path.join("/"))
        }
    }
}
