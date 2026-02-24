# Audio Capture Design

## The Core Constraint

In real-time audio programming there is one ruthless rule: **you must never block the audio callback thread.**

When `cpal` opens a stream on Windows (WASAPI) or Ubuntu (ALSA/PulseAudio), it spawns a high-priority OS thread. This thread fires a callback thousands of times per second. If your code in that callback waits for a lock, allocates memory on the heap, or blocks for I/O, you miss the OS deadline. The result: audio dropouts, crackling, and corrupted WAV files.

The problem: transport an endless stream of `f32` samples from this volatile, high-priority thread to a safe place in memory, while a separate thread watches the keyboard for the Enter key.

## Design 1: Naive Shared State (`Arc<Mutex<Vec<f32>>>`)

Create a dynamically sizing `Vec`, wrap it in a thread-safe `Mutex`, share it across threads, lock it every time a sample arrives.

**Flow:**
1. Main thread creates `Arc<Mutex<Vec<f32>>>`.
2. Clone of the `Arc` is passed into the cpal callback closure.
3. Every callback: acquire lock, append samples.
4. Main thread blocks on `stdin.read_line()`. On Enter: stop stream, acquire lock, process vector.

**Pros:**
- Uses only the Rust standard library.
- Mental model is simple.

**Cons:**
- **Priority Inversion**: If the main thread holds the lock when the audio thread needs it, the audio thread stalls completely.
- **Heap Allocation in Callback**: As the `Vec` grows, it reallocates memory. Memory allocation is a blocking OS operation that drops frames.

**Verdict: Rejected.** Will cause random audio glitches depending on OS scheduler behavior.

---

## Design 2: Chunked MPSC Channel (`crossbeam::channel`)

Use message passing. The audio thread acts as a pure producer, throwing data over a wall to a consumer thread.

**Flow:**
1. Instantiate `crossbeam_channel::unbounded()`.
2. The cpal callback takes the `Sender`.
3. Callback accumulates a small fixed-size chunk (e.g., 512 frames) and sends it.
4. A dedicated background thread loops on the `Receiver`, pulling chunks into a `Vec<f32>`.
5. Main thread waits for Enter, signals consumer to stop, grabs the final `Vec`.

**Pros:**
- No mutex contention. Audio thread fires and forgets.
- Consistent behavior across Windows and Linux schedulers.
- Downsampling can run in the receiver thread, protecting the cpal callback from DSP math.

**Cons:**
- Sending dynamic chunks still involves some underlying allocation and pointer management.
- Chunk size tuning required: too small starves the receiver, too large wastes CPU on channel signaling.

**Verdict: Acceptable but not optimal.**

---

## Design 3: Lock-Free Ring Buffer (`ringbuf` crate) — SELECTED

The gold standard for real-time DSP and audio engineering. A Single-Producer, Single-Consumer (SPSC) lock-free ring buffer allocates all memory upfront. The producer pushes samples to the head; the consumer pulls from the tail.

**Flow:**
1. Main thread allocates `ringbuf::HeapRb<f32>` (e.g., ~60 seconds of audio, ~3MB).
2. Buffer is split into a `Producer` and a `Consumer`.
3. The `Producer` moves into the cpal callback. Executes `push_slice()` — a simple memory copy.
4. Because it's lock-free and pre-allocated, `push_slice()` runs in constant time O(1) with zero OS interaction.
5. A background thread periodically checks the `Consumer`, drains samples, downsamples to 16kHz, and stores in the final payload buffer.
6. Main thread waits for Enter, drops the cpal stream, drains remaining ring buffer, fires HTTP request.

**Pros:**
- **Absolute real-time safety**: Zero locks, zero mutexes, zero heap allocations in the audio path.
- **Extreme efficiency**: Pushing data is just updating an atomic index and moving memory.
- **Predictable memory footprint**: Pre-allocated exactly once at startup.

**Cons:**
- More complex: managing two atomic indexes, a pre-allocated buffer, and a separate drain thread.
- Buffer overrun risk: if the consumer stalls and the ring buffer fills, the producer drops frames. Must size the buffer large enough to absorb OS hiccups.

**Verdict: Selected for DictateYourTerms.** Dropping a single syllable because the OS ran a background scan and paused a Mutex is unacceptable for dictating complex architectural intent.
