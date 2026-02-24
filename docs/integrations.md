# Editor and Terminal Integrations

## Neovim

Pipe dictation directly into the active Neovim buffer via a Lua keymap in `init.lua`:

```lua
vim.keymap.set({'n', 'i'}, '<leader>v', function()
    print("Listening (Press Enter in terminal to stop)...")
    -- Spawn the stt-cli process
    vim.fn.system('dyt --record')

    -- Pull the result from the system clipboard
    local transcript = vim.fn.getreg('+')

    -- Insert the text at the cursor position
    vim.api.nvim_put({transcript}, 'c', true, true)
    print("Dictation inserted.")
end, { desc = "Voice Dictation via local Whisper daemon" })
```

## Terminal / WezTerm / Tmux

Bind the same recording command to a hotkey in your terminal emulator or Tmux config. The CLI captures audio, transcribes, and populates the clipboard. You then paste with your normal keybinding.

```bash
dyt --record
```

## PowerShell (Windows)

Set up a custom PSReadLine key handler in `$PROFILE` to invoke `dyt --record`. Hold the macro key, dictate your prompt, release, and the transcribed text populates the active command line buffer ready for Enter.
