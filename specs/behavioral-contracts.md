# Behavioral Contracts

Declarative specifications defining the state machine and boundary conditions of the architecture.

## Feature: Daemon State Management and Inference

### Scenario: Successful transcription of valid audio payload

```gherkin
Given the dyt-daemon is running on 127.0.0.1:3030
And the configured ModelProvider is successfully loaded into memory
When a POST request is received at /transcribe
And the request body contains a valid 16kHz mono WAV file
Then the daemon acquires the mutex lock on the ModelProvider
And the audio data is processed synchronously
And the daemon returns an HTTP 200 OK status
And the response body contains the plain text transcription
And the mutex lock is released
```

### Scenario: Graceful handling of invalid audio formats

```gherkin
Given the dyt-daemon is running
When a POST request is received at /transcribe with an invalid or corrupt audio payload
Then the daemon rejects the payload before inference begins
And the daemon returns an HTTP 400 Bad Request status
And the daemon remains alive and ready for the next request
```

## Feature: CLI Hardware Capture and Injection

### Scenario: End-to-end dictation capture and clipboard population

```gherkin
Given the dyt-cli is executed with the --record flag
When the application binds to the default OS audio input device via cpal
Then the CLI begins recording audio samples to an in-memory buffer
When the user inputs a termination signal (e.g., standard input newline)
Then the audio stream is instantly halted
And the buffer is encoded into a WAV payload
And the payload is transmitted via HTTP POST to the dyt-daemon
When the dyt-cli receives the transcribed string
Then the string is injected into the operating system's primary clipboard
And the dyt-cli process terminates with exit code 0
```

## Feature: Neovim Plugin Integration

### Scenario: Successful end-to-end dictation from Neovim

```gherkin
Given the dyt-daemon is running
And the dyt binary is on PATH in the Neovim environment
And the Neovim plugin is loaded
When the user presses the configured keymap (<leader>v by default)
Then a floating terminal window opens running dyt --record
When the user speaks and presses Enter in the floating terminal
Then the terminal process exits with code 0
And the floating window closes automatically
And the plugin reads the system clipboard
And the transcript is inserted at the cursor position in the originating buffer
And a success notification is shown if notify = true
```

### Scenario: Graceful handling of missing dyt binary

```gherkin
Given the dyt binary is not on PATH in the Neovim environment
When the user presses the configured keymap
Then a floating terminal window opens
And termopen raises an error (E475) which the plugin catches via pcall
And the floating window is closed
And the plugin state is fully reset
And an error notification is shown: "Failed to start dyt. Is it installed and on PATH?"
And the plugin is immediately ready for another invocation
```

### Scenario: Graceful handling of daemon not running

```gherkin
Given the dyt binary is on PATH
And the dyt-daemon is not running
When the user presses the configured keymap
Then a floating terminal window opens
And dyt exits with a non-zero exit code
And the floating window closes automatically
And an error notification is shown containing the exit code
And the plugin state is fully reset
```

### Scenario: Re-entrancy guard

```gherkin
Given a recording session is already active (floating terminal is open)
When the user presses the configured keymap a second time
Then no second floating terminal is opened
And no second dyt process is spawned
And a warning notification is shown: "Already recording."
```
