# Behavioral Contracts

Declarative specifications defining the state machine and boundary conditions of the architecture.

## Feature: Daemon State Management and Inference

### Scenario: Successful transcription of valid audio payload

```gherkin
Given the stt-daemon is running on 127.0.0.1:3030
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
Given the stt-daemon is running
When a POST request is received at /transcribe with an invalid or corrupt audio payload
Then the daemon rejects the payload before inference begins
And the daemon returns an HTTP 400 Bad Request status
And the daemon remains alive and ready for the next request
```

## Feature: CLI Hardware Capture and Injection

### Scenario: End-to-end dictation capture and clipboard population

```gherkin
Given the stt-cli is executed with the --record flag
When the application binds to the default OS audio input device via cpal
Then the CLI begins recording audio samples to an in-memory buffer
When the user inputs a termination signal (e.g., standard input newline)
Then the audio stream is instantly halted
And the buffer is encoded into a WAV payload
And the payload is transmitted via HTTP POST to the stt-daemon
When the stt-cli receives the transcribed string
Then the string is injected into the operating system's primary clipboard
And the stt-cli process terminates with exit code 0
```
