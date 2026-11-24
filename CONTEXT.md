# Clip2Pod

Clip2Pod turns web content into podcast episodes that a phone can subscribe to over the local network.

## Language

### Modes

**Mode**:
One of the two independent ways Clip2Pod produces episodes, each with its own queue, log and feed. There are exactly two: Narrate and Rip.
_Avoid_: tab, pipeline, engine

**Narrate**:
The Mode that turns captured text into a spoken episode.
_Avoid_: TTS mode, read

**Rip**:
The Mode that turns a video URL into an audio episode.
_Avoid_: download mode, yt-dlp mode

### Jobs

**Job**:
One requested episode waiting in, or moving through, a Mode's queue.
_Avoid_: task, item, render

**Settled job**:
A Job that has reached Done, Failed or Cancelled, and will not change again unless it is retried.
_Avoid_: finished job, terminal job

**Log row**:
The permanent record of one Job event in a Mode's log, such as a Job settling (and, in Rip, a Job being queued).
_Avoid_: history entry, log line

**Lamp**:
The on-air indicator for one Mode: idle, queued (with a count), or on air.
_Avoid_: status light, badge
