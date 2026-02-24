# Editor and Terminal Integrations

## Neovim

A first-party Lua plugin ships at `editors/nvim/` in this repository. It opens a floating terminal running `dyt --record`, waits for you to speak and press Enter, then auto-closes the float and inserts the transcript at the cursor. Works from both normal and insert mode.

### Installation

**lazy.nvim** — point at the subdirectory inside a local clone:

```lua
{
  dir = '/path/to/dictate-your-terms/editors/nvim',
}
```

**packer.nvim**:

```lua
use { '/path/to/dictate-your-terms/editors/nvim' }
```

**vim-plug**:

```vim
Plug '/path/to/dictate-your-terms/editors/nvim'
```

**Manual (any plugin manager or bare Neovim)**:

```lua
vim.opt.runtimepath:append('/path/to/dictate-your-terms/editors/nvim')
```

The plugin auto-initialises with defaults on startup via `plugin/dyt.lua`. If you call `setup()` yourself before that shim fires, the shim is a no-op.

### Configuration

Call `require('dyt').setup(opts)` in your Neovim config. All keys are optional; omit any to keep the default.

```lua
require('dyt').setup({
  keymap     = '<leader>v',              -- trigger in normal and insert mode
  daemon     = 'http://127.0.0.1:3030',  -- dyt-daemon address
  win_width  = 0.5,                      -- float width as fraction of editor width
  win_height = 10,                       -- float height in rows
  border     = 'rounded',                -- any nvim_open_win border style
  notify     = true,                     -- emit vim.notify status messages
})
```

| Option       | Type    | Default                       | Description                                          |
|--------------|---------|-------------------------------|------------------------------------------------------|
| `keymap`     | string  | `'<leader>v'`                 | Key sequence bound in both normal and insert mode    |
| `daemon`     | string  | `'http://127.0.0.1:3030'`     | HTTP base URL of the running `dyt-daemon`            |
| `win_width`  | number  | `0.5`                         | Float width as a fraction of the current editor width|
| `win_height` | number  | `10`                          | Float height in absolute rows                        |
| `border`     | string  | `'rounded'`                   | Border style passed to `nvim_open_win`               |
| `notify`     | boolean | `true`                        | Whether to emit `vim.notify` status messages         |

### Prerequisites

- `dyt-daemon` must be running before pressing the keymap.
- The `dyt` binary must be on `PATH` in the environment that launches Neovim. If it is not, the plugin shows `[dyt] Failed to start dyt. Is it installed and on PATH?` and cleans up without crashing.

### Behavior

1. Pressing the keymap opens a floating terminal and runs `dyt --record`.
2. Speak. Press Enter in the terminal to stop recording.
3. The float closes automatically.
4. The plugin reads the system clipboard and inserts the transcript at the cursor position.
5. If `notify = true`, a message is shown on success or on non-zero exit.
6. A re-entrancy guard prevents a second invocation while the float is open.

### Customising the keymap

To use a different key sequence, pass `keymap` to `setup()`:

```lua
require('dyt').setup({ keymap = '<C-r>' })
```

To suppress the default binding and manage the keymap yourself, pass `keymap = false`:

```lua
require('dyt').setup({ keymap = false })
```

## Terminal / WezTerm / Tmux

Bind the same recording command to a hotkey in your terminal emulator or Tmux config. The CLI captures audio, transcribes, and populates the clipboard. You then paste with your normal keybinding.

```bash
dyt --record
```

## PowerShell (Windows)

Set up a custom PSReadLine key handler in `$PROFILE` to invoke `dyt --record`. Hold the macro key, dictate your prompt, release, and the transcribed text populates the active command line buffer ready for Enter.
