#!/usr/bin/env python3
"""Generate procedural, loop-safe WAV music stems for LitTCG.

Each stem is built from integer harmonics of a base frequency whose period
fits evenly into the sample count, so the loop can play gaplessly.

This is the musical equivalent of the FACES protocol: language data (words,
elements, game state) becomes sound. For now we generate three core stems:
menu theme, world ambience, and battle tension.
"""

import math
import os
import struct
import wave

SAMPLE_RATE = 44100
OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "..", "assets", "sounds")
MAX_AMP = 32767


def generate_track(
    path: str,
    base_freq: float,
    duration: float,
    harmonics: list[tuple[int, float]],
    envelope_cycles: int = 1,
    tremolo_cycles: int = 0,
) -> None:
    """Write a single looping WAV track.

    `harmonics` is a list of (harmonic_index, relative_amplitude).
    The envelope is a one- or multi-cycle sinusoid over the loop length,
    which is safe to loop because its value and slope match at start/end.
    """
    os.makedirs(os.path.dirname(path), exist_ok=True)

    base_period = SAMPLE_RATE / base_freq
    # Force integer number of base cycles so the fundamental loops cleanly.
    cycles = round(duration * base_freq)
    total_samples = int(round(cycles * base_period))
    loop_duration = total_samples / SAMPLE_RATE

    total_harmonic_amp = sum(amp for _, amp in harmonics) or 1.0

    samples = []
    for i in range(total_samples):
        t = i / SAMPLE_RATE
        signal = 0.0
        for h, rel_amp in harmonics:
            freq = base_freq * h
            signal += (rel_amp / total_harmonic_amp) * math.sin(2.0 * math.pi * freq * t)

        # Global envelope: starts and ends at a mid-level, swelling in the middle.
        # Using `envelope_cycles` sine waves over the loop keeps start/end continuous.
        base_env = 0.7
        swell = 0.3
        env = base_env + swell * math.sin(
            2.0 * math.pi * envelope_cycles * i / total_samples
        )

        # Optional faster tremolo for tension layers.
        if tremolo_cycles > 0:
            trem = 0.5 + 0.5 * math.sin(
                2.0 * math.pi * tremolo_cycles * i / total_samples
            )
            env *= 0.8 + 0.2 * trem

        sample = int(MAX_AMP * signal * env * 0.95)
        # Clip just in case of floating-point drift.
        sample = max(-MAX_AMP, min(MAX_AMP, sample))
        samples.append(struct.pack("<h", sample))

    with wave.open(path, "w") as f:
        f.setnchannels(1)
        f.setsampwidth(2)
        f.setframerate(SAMPLE_RATE)
        f.setnframes(len(samples))
        f.writeframes(b"".join(samples))

    print(f"Wrote {path}: {loop_duration:.3f}s, {len(samples)} samples")


def main() -> None:
    out_dir = OUTPUT_DIR
    print(f"Generating music stems into {out_dir}")

    # Menu: calm, low, slow-swell major-ish drone.
    generate_track(
        os.path.join(out_dir, "music_menu.wav"),
        base_freq=44100 / 600,  # ~73.5 Hz, low A2/D2-ish root
        duration=4.0,
        harmonics=[(1, 0.35), (2, 0.25), (3, 0.20), (4, 0.15), (5, 0.05)],
        envelope_cycles=1,
    )

    # World ambience: warm pad with a slow single swell.
    generate_track(
        os.path.join(out_dir, "music_world.wav"),
        base_freq=44100 / 480,  # ~91.9 Hz, low F#1/G1-ish
        duration=5.0,
        harmonics=[(1, 0.30), (2, 0.25), (3, 0.20), (4, 0.15), (5, 0.10)],
        envelope_cycles=1,
    )

    # Battle: darker, odd harmonics, fast tremolo.
    generate_track(
        os.path.join(out_dir, "music_battle.wav"),
        base_freq=44100 / 400,  # ~110.25 Hz, A2-ish
        duration=3.0,
        harmonics=[(1, 0.30), (3, 0.25), (5, 0.20), (7, 0.15), (9, 0.10)],
        envelope_cycles=1,
        tremolo_cycles=6,
    )


if __name__ == "__main__":
    main()
