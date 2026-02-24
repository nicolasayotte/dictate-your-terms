local M = {}

M._setup_called  = false
M._config        = {}
M._recording     = false
M._origin_win    = nil
M._origin_mode   = nil
M._float_win     = nil
M._float_buf     = nil

local defaults = {
  keymap     = '<leader>v',
  daemon     = 'http://127.0.0.1:3030',
  win_width  = 0.5,
  win_height = 10,
  border     = 'rounded',
  notify     = true,
}

local function notify(msg, level)
  if M._config.notify then
    vim.notify('[dyt] ' .. msg, level)
  end
end

local function float_opts()
  local ui = vim.api.nvim_list_uis()[1]
  -- Guard: headless Neovim (CI, --headless) has no UI; fail fast with a clear message.
  if not ui then
    error('[dyt] No UI available — cannot open float window')
  end
  local width  = math.floor(ui.width  * M._config.win_width)
  local height = M._config.win_height
  local row    = math.floor((ui.height - height) / 2)
  local col    = math.floor((ui.width  - width)  / 2)
  return {
    relative = 'editor',
    width    = math.max(width, 20),
    height   = math.max(height, 3),
    row      = row,
    col      = col,
    style    = 'minimal',
    border   = M._config.border,
  }
end

local function reset_state()
  M._recording   = false
  M._origin_win  = nil
  M._origin_mode = nil
  M._float_win   = nil
  M._float_buf   = nil
end

local function close_float()
  if M._float_win and vim.api.nvim_win_is_valid(M._float_win) then
    vim.api.nvim_win_close(M._float_win, true)
  end
end

local function on_exit(_, exit_code, _)
  vim.schedule(function()
    close_float()

    if exit_code ~= 0 then
      notify(
        string.format('dyt exited with code %d. Is the daemon running?', exit_code),
        vim.log.levels.ERROR
      )
      reset_state()
      return
    end

    local transcript = vim.fn.getreg('+')

    if transcript == '' then
      notify('Transcription returned empty text.', vim.log.levels.WARN)
      reset_state()
      return
    end

    local origin = M._origin_win
    if origin and vim.api.nvim_win_is_valid(origin) then
      vim.api.nvim_set_current_win(origin)
    end

    -- Reset state before nvim_put so a paste failure (e.g. read-only buffer)
    -- cannot strand _recording = true and soft-lock the plugin.
    reset_state()

    local lines = vim.split(transcript, '\n', { plain = true })
    vim.api.nvim_put(lines, 'c', true, true)

    notify('Dictation inserted.', vim.log.levels.INFO)
  end)
end

local function start_dictation()
  if M._recording then
    notify('Already recording.', vim.log.levels.WARN)
    return
  end

  M._origin_win  = vim.api.nvim_get_current_win()
  M._origin_mode = vim.api.nvim_get_mode().mode

  if M._origin_mode == 'i' or M._origin_mode == 'ic' or M._origin_mode == 'ix' then
    vim.api.nvim_feedkeys(
      vim.api.nvim_replace_termcodes('<Esc>', true, false, true),
      'n',
      false
    )
  end

  local buf = vim.api.nvim_create_buf(false, true)
  M._float_buf = buf

  local win = vim.api.nvim_open_win(buf, true, float_opts())
  M._float_win = win

  vim.wo[win].number         = false
  vim.wo[win].relativenumber = false
  vim.wo[win].signcolumn     = 'no'

  local cmd = { 'dyt', '--record', '--daemon', M._config.daemon }
  local ok, job_id = pcall(vim.fn.termopen, cmd, { on_exit = on_exit })

  if not ok or job_id <= 0 then
    notify('Failed to start dyt. Is it installed and on PATH?', vim.log.levels.ERROR)
    close_float()
    reset_state()
    return
  end

  M._recording = true
  notify('Recording... press Enter in the terminal to stop.', vim.log.levels.INFO)

  vim.cmd('startinsert')
end

local function register_keymap()
  local km = M._config.keymap
  if not km or km == '' then
    return
  end
  local opts = { noremap = true, silent = true, desc = 'DictateYourTerms: voice dictation' }
  vim.keymap.set({ 'n', 'i' }, km, start_dictation, opts)
end

function M.setup(opts)
  M._config       = vim.tbl_deep_extend('force', defaults, opts or {})
  M._setup_called = true
  register_keymap()
end

return M
