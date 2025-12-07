# twiggle
---
<img width="1920" height="720" alt="TwiggleV2" src="https://github.com/user-attachments/assets/4072ae58-9925-45cb-87e6-6a98cec75edc" />

---
twiggle is a fast directory traversal tool which allows you to move between directories using keybinds.

## Installation
To install twiggle, it is currently necessary to have `cargo` set up. For more information, look at the [rustup](https://rust-lang.org/tools/install/) installation guide.
### Git + Cargo
You can install this repository directly via `cargo`:
```
cargo install --git https://github.com/MarwinLinke/twiggle.git
```
Alternatively, if you encounter problems or want to have the repository at a specific location, use this:
```
git clone https://github.com/MarwinLinke/twiggle.git
cd twiggle
cargo install --path .
```
In both cases, the executable should be created in `.cargo/bin`.

### Setup
> [!IMPORTANT]
> When executing `twiggle` directly, no actual directories will be changed. For full functionality, an alias or function must be created in the shell configuration file.

Depending on your shell, you can add one of the following:

#### Bash / Zsh
Add this to your `~/.bashrc` or `~/.zshrc`:
```
alias twg='cd "$(twiggle --icons)"'
```

#### Fish
Add this to your `~/.config/fish/config.fish`:
```
function twg
    cd (twiggle --icons)
end
```

It is also recommended to choose your own alias for quick activation, as well as to look at the flags for the best experience.

### Flags
Currently, the following flags are available:

| Flag          | Description                                                      |
|---------------|------------------------------------------------------------------|
| `--icons`     | Enables icons (a nerd font is needed for icons to be displayed). |
| `--no-colors` | Disables all colors.                                             |
| `--debug`     | Displays custom debug messages.                                  |
| `--clear`     | Clears the output after closing.                                 |
| `--hide`      | Hides dot files and directories.                                 |

Remember to add these to your shell configuration file instead of behind the alias.

## Usage
### Useful key binds

| Key bind      | Action                               |
|---------------|--------------------------------------|
| `<Enter>`     | Close and move to directory.         |
| `<Esc>`       | Close and cancel.                    |
| `<Backspace>` | Go into the parent directory.        |
| `<~>`         | Go to the home directory.            |
| `<C-b>`       | Go page backward.                    |
| `<C-f>`       | Go page forward.                     |
| `<C-n>`       | Rotate through pages.                |
| `<C-s>`       | Show/Hide dot files and directories. |

## Special Thanks
Displaying icons per file type was made possible with the logic inside `icons.rs` which was written by the [eza-community](https://github.com/eza-community/eza).
