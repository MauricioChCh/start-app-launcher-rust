# M-Launcher

A Linux multi application launcher written in Rust that allows you to start groups of applications with a TUI (Terminal User Interface).

## Description

Launcher is a tool that facilitates the startup of multiple applications organized into groups. Ideal for automating your work environment setup at system boot, allowing you to choose which set of applications to launch based on the task you're about to perform.

## Features

- Intuitive TUI interface using Ratatui
- JSON-based configuration
- Multiple applications per group execution
- Support for simple and complex commands (with shell)
- Applications run independently (don't die when launcher closes)
- Keyboard navigation (vim-style and arrows)

## Installation

### Build from source

```bash
cargo build --release
```

### Install system-wide

```bash
sudo install -Dm755 target/release/launcher /usr/local/bin/launcher
```

## Configuration

The launcher searches for the configuration file in the following order:

1. `./launcher.json` (current directory)
2. `~/.config/launcher/config.json`
3. `/etc/launcher/config.json`

### Configuration file structure

```json
{
  "groups": [
    {
      "name": "Development",
      "apps": [
        {
          "name": "VSCode",
          "command": "code",
          "args": []
        },
        {
          "name": "Terminal",
          "command": "kitty",
          "args": []
        }
      ]
    },
    {
      "name": "Docker Services",
      "apps": [
        {
          "name": "Start Docker Containers",
          "command": "sh",
          "args": ["-c", "docker start $(docker ps -aq)"],
          "use_shell": true
        }
      ]
    }
  ]
}
```

### Configuration options per application

- `name`: Descriptive name of the application
- `command`: Command to execute
- `args`: Array of arguments for the command (optional)
- `use_shell`: Boolean indicating whether to use `sh -c` to execute complex commands (optional)

### Configuration example

Create the configuration directory:

```bash
mkdir -p ~/.config/launcher/
```

Create the configuration file:

```bash
nano ~/.config/launcher/config.json
```

## Usage

Run the launcher:

```bash
launcher
```

### Controls

- Up/Down arrows or `k`/`j`: Navigate between groups
- Enter: Select and execute group
- `q` or Esc: Exit without executing

## Autostart

### For systems with autostart support (KDE, GNOME, etc.)

Create a `.desktop` file in `~/.config/autostart/`:

```bash
nano ~/.config/autostart/launcher.desktop
```

File content:

```desktop
[Desktop Entry]
Type=Application
Exec=kitty --class=launcher-term --title="Launcher" -e launcher
Name=Launcher
Comment=Auto start Launcher in Kitty
X-KDE-autostart-after=panel
```

Note: Adjust the `Exec` command according to your preferred terminal emulator.

## Requirements

- Rust 1.70 or higher
- Linux/Unix operating system

## Dependencies

- ratatui: TUI framework
- crossterm: Terminal handling
- serde: Serialization/deserialization
- serde_json: JSON handling

## License

Open source personal project.