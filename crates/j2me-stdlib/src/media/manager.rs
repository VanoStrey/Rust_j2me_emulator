use super::midi::{MidiSequence, MidiSynthesizer};
use super::wav::WavDecoder;
use std::collections::VecDeque;

pub const STATE_UNREALIZED: i32 = 100;
pub const STATE_REALIZED: i32 = 200;
pub const STATE_PREFETCHED: i32 = 300;
pub const STATE_STARTED: i32 = 400;
pub const STATE_CLOSED: i32 = 0;

#[derive(Debug, Clone, Copy)]
pub struct DcBlockingFilter {
    pub r: f32,
    pub prev_x: f32,
    pub prev_y: f32,
}

impl DcBlockingFilter {
    pub fn new(r: f32) -> Self {
        Self {
            r,
            prev_x: 0.0,
            prev_y: 0.0,
        }
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        if !x.is_finite() {
            self.prev_x = 0.0;
            self.prev_y = 0.0;
            return 0.0;
        }
        let mut y = x - self.prev_x + self.r * self.prev_y;
        if !y.is_finite() || y.abs() < 1e-12 {
            y = 0.0;
        }
        self.prev_x = x;
        self.prev_y = y;
        y
    }

    pub fn reset(&mut self) {
        self.prev_x = 0.0;
        self.prev_y = 0.0;
    }
}

impl Default for DcBlockingFilter {
    fn default() -> Self {
        Self::new(0.995)
    }
}

#[derive(Debug, Clone)]
pub struct AudioRingBuffer {
    samples: VecDeque<f32>,
    pub sample_rate: u32,
    max_capacity: usize,
    pub generated_samples: u64,
    pub consumed_samples: u64,
    pub dropped_samples: u64,
    pub underrun_count: u64,
    pub overflow_count: u64,
    pub peak_level: f32,
    pub clipped_samples: u64,
    pub total_energy: f64,
    pub dc_blocker: DcBlockingFilter,
}

impl AudioRingBuffer {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            samples: VecDeque::with_capacity((sample_rate * 2) as usize),
            sample_rate,
            max_capacity: (sample_rate * 2) as usize,
            generated_samples: 0,
            consumed_samples: 0,
            dropped_samples: 0,
            underrun_count: 0,
            overflow_count: 0,
            peak_level: 0.0,
            clipped_samples: 0,
            total_energy: 0.0,
            dc_blocker: DcBlockingFilter::new(0.995),
        }
    }

    pub fn write_sample(&mut self, raw_sample: f32) {
        self.generated_samples += 1;
        let sample = self.dc_blocker.process(raw_sample);
        self.total_energy += (sample * sample) as f64;
        let abs = sample.abs();
        if abs > self.peak_level {
            self.peak_level = abs;
        }
        if abs >= 0.99 {
            self.clipped_samples += 1;
        }
        if self.samples.len() >= self.max_capacity {
            self.samples.pop_front();
            self.dropped_samples += 1;
            self.overflow_count += 1;
        }
        self.samples.push_back(sample);
    }

    pub fn get_and_decay_peak(&mut self) -> f32 {
        let p = self.peak_level;
        self.peak_level *= 0.95;
        p
    }

    pub fn read_samples(&mut self, dest: &mut [f32]) -> usize {
        let count = dest.len().min(self.samples.len());
        if dest.len() > self.samples.len() {
            self.underrun_count += 1;
        }
        for item in dest.iter_mut().take(count) {
            *item = self.samples.pop_front().unwrap_or(0.0);
        }
        self.consumed_samples += count as u64;
        count
    }

    /// Computes (peak, RMS, DC offset) for an audio sample slice.
    pub fn compute_metrics(samples: &[f32]) -> (f32, f32, f32) {
        if samples.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let mut peak = 0.0f32;
        let mut sum_sq = 0.0f64;
        let mut sum = 0.0f64;
        for &s in samples {
            let abs = s.abs();
            if abs > peak {
                peak = abs;
            }
            sum += s as f64;
            sum_sq += (s as f64) * (s as f64);
        }
        let n = samples.len() as f64;
        let dc_offset = (sum / n) as f32;
        let rms = (sum_sq / n).sqrt() as f32;
        (peak, rms, dc_offset)
    }

    pub fn synthesize_tone(&mut self, note: i32, duration_ms: i32, volume: i32) {
        if duration_ms <= 0 {
            return;
        }
        let freq = 440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0);
        let num_samples = (self.sample_rate as f32 * (duration_ms as f32 / 1000.0)) as usize;
        let amp = (volume.clamp(0, 100) as f32 / 100.0) * 0.35;

        let attack_samples = (self.sample_rate as f32 * 0.010).min(num_samples as f32 / 4.0) as usize;
        let release_samples = (self.sample_rate as f32 * 0.015).min(num_samples as f32 / 4.0) as usize;

        for i in 0..num_samples {
            let env = if attack_samples > 0 && i < attack_samples {
                i as f32 / attack_samples as f32
            } else if release_samples > 0 && i + release_samples >= num_samples {
                (num_samples - 1 - i) as f32 / release_samples as f32
            } else {
                1.0
            };
            let t = i as f32 / self.sample_rate as f32;
            let sample = (amp * env * (2.0 * std::f32::consts::PI * freq * t).sin()).tanh();
            self.write_sample(sample);
        }
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

pub struct Player {
    pub id: i32,
    pub state: i32,
    pub content_type: String,
    pub volume: i32,
    pub media_data: Vec<u8>,
    pub loop_count: i32,
    pub sample_rate: u32,
    pub wav_decoder: Option<WavDecoder>,
    pub wav_play_head: usize,
    pub midi_synth: Option<MidiSynthesizer>,
}

impl Player {
    pub fn new(id: i32, content_type: &str, media_data: Vec<u8>, sample_rate: u32) -> Self {
        let is_midi = content_type.contains("midi")
            || (media_data.len() >= 4 && &media_data[0..4] == b"MThd");
        let is_wav =
            content_type.contains("wav") || (media_data.len() >= 4 && &media_data[0..4] == b"RIFF");

        let midi_synth = if is_midi {
            match MidiSequence::parse(&media_data) {
                Ok(seq) => Some(MidiSynthesizer::new(seq, sample_rate)),
                Err(_) => None,
            }
        } else {
            None
        };

        let wav_decoder = if is_wav {
            WavDecoder::parse(&media_data, sample_rate).ok()
        } else {
            None
        };

        Self {
            id,
            state: STATE_UNREALIZED,
            content_type: content_type.to_string(),
            volume: 100,
            media_data,
            loop_count: 1,
            sample_rate,
            wav_decoder,
            wav_play_head: 0,
            midi_synth,
        }
    }

    pub fn realize(&mut self) {
        if self.state == STATE_UNREALIZED {
            self.state = STATE_REALIZED;
        }
    }

    pub fn prefetch(&mut self) {
        self.realize();
        if self.state == STATE_REALIZED {
            self.state = STATE_PREFETCHED;
        }
    }

    pub fn start(&mut self, _audio: &mut AudioRingBuffer) {
        self.prefetch();
        self.state = STATE_STARTED;

        if let Some(ref mut synth) = self.midi_synth {
            synth.loop_count = self.loop_count;
            synth.volume = self.volume;
            synth.reset();
            synth.start();
        }

        // WAV is rendered by `render_audio_into_buffer`; starting must only
        // rewind the play head. Writing samples here as well would emit the
        // clip twice and leave the head advanced on the next start.
        if self.wav_decoder.is_some() {
            self.wav_play_head = 0;
        }
    }

    pub fn stop(&mut self) {
        if self.state == STATE_STARTED {
            self.state = STATE_PREFETCHED;
            if let Some(ref mut synth) = self.midi_synth {
                synth.stop();
            }
        }
    }

    pub fn close(&mut self) {
        self.stop();
        self.state = STATE_CLOSED;
    }

    pub fn set_loop_count(&mut self, count: i32) {
        self.loop_count = count;
        if let Some(ref mut synth) = self.midi_synth {
            synth.loop_count = count;
        }
    }

    pub fn set_volume(&mut self, level: i32) {
        self.volume = level.clamp(0, 100);
        if let Some(ref mut synth) = self.midi_synth {
            synth.volume = self.volume;
        }
    }

    /// Current playback position in milliseconds.
    pub fn media_time(&self) -> i64 {
        if let Some(ref synth) = self.midi_synth {
            return synth.media_time_ms();
        }
        if let Some(ref wav) = self.wav_decoder {
            if wav.samples.is_empty() || self.sample_rate == 0 {
                return 0;
            }
            return (self.wav_play_head as f64 / self.sample_rate as f64 * 1000.0) as i64;
        }
        0
    }

    /// Seeks to `ms`, clamped to the clip length, and returns the position the
    /// player actually landed on (as `setMediaTime` is required to do).
    pub fn set_media_time(&mut self, ms: i64) -> i64 {
        let ms = ms.max(0);

        if let Some(ref mut synth) = self.midi_synth {
            synth.seek_ms(ms);
            return synth.media_time_ms();
        }

        if let Some(ref wav) = self.wav_decoder {
            if self.sample_rate == 0 {
                return 0;
            }
            let index = ((ms as f64 / 1000.0) * self.sample_rate as f64) as usize;
            self.wav_play_head = index.min(wav.samples.len());
            return self.media_time();
        }

        0
    }

    pub fn render_audio_samples(&mut self, count: usize, dest: &mut Vec<f32>) {
        if self.state != STATE_STARTED {
            return;
        }

        if let Some(ref mut synth) = self.midi_synth {
            synth.render_samples(count, dest);
        } else if let Some(ref wav) = self.wav_decoder {
            let vol = self.volume.clamp(0, 100) as f32 / 100.0;
            let available = wav.samples.len().saturating_sub(self.wav_play_head);
            let chunk_size = count.min(available);
            let total_len = wav.samples.len();
            let fade_len = ((self.sample_rate as f32 * 0.0025).round() as usize).clamp(8, (total_len / 4).max(8));

            dest.reserve(chunk_size);
            for i in 0..chunk_size {
                let current_idx = self.wav_play_head + i;
                let mut s = wav.samples[current_idx];

                // Smooth loop boundary crossfade and end-of-clip fade-out to prevent pops
                if total_len > fade_len * 2 && current_idx + fade_len >= total_len {
                    let remaining = total_len - 1 - current_idx;
                    let fade = remaining as f32 / fade_len as f32;
                    if self.loop_count < 0 || self.loop_count > 1 {
                        let start_idx = fade_len.saturating_sub(remaining);
                        let start_s = if start_idx < wav.samples.len() { wav.samples[start_idx] } else { 0.0 };
                        s = s * fade + start_s * (1.0 - fade);
                    } else {
                        s *= fade;
                    }
                }

                dest.push(s * vol);
            }
            self.wav_play_head += chunk_size;

            if self.wav_play_head >= wav.samples.len() {
                if self.loop_count < 0 || self.loop_count > 1 {
                    if self.loop_count > 1 {
                        self.loop_count -= 1;
                    }
                    self.wav_play_head = 0;
                } else {
                    self.state = STATE_PREFETCHED;
                }
            }
        }
    }

    pub fn render_audio_into_buffer(&mut self, count: usize, audio: &mut AudioRingBuffer) {
        let mut samples = Vec::with_capacity(count);
        self.render_audio_samples(count, &mut samples);
        for s in samples {
            audio.write_sample(s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav_bytes(rate: u32, frames: usize, amplitude: f32) -> Vec<u8> {
        let mut pcm = Vec::new();
        for i in 0..frames {
            let v = ((i as f32 * 0.05).sin() * amplitude * 32767.0) as i16;
            pcm.extend_from_slice(&v.to_le_bytes());
        }
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&rate.to_le_bytes());
        out.extend_from_slice(&(rate * 2).to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
        out.extend_from_slice(&pcm);
        out
    }

    fn midi_bytes() -> Vec<u8> {
        let track = vec![
            0x00, 0x90, 0x3C, 0x64, 0x60, 0x80, 0x3C, 0x00, 0x00, 0xFF, 0x2F, 0x00,
        ];
        let mut out = Vec::new();
        out.extend_from_slice(b"MThd");
        out.extend_from_slice(&6u32.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&1u16.to_be_bytes());
        out.extend_from_slice(&96u16.to_be_bytes());
        out.extend_from_slice(b"MTrk");
        out.extend_from_slice(&(track.len() as u32).to_be_bytes());
        out.extend_from_slice(&track);
        out
    }

    fn wav_player() -> Player {
        Player::new(1, "audio/x-wav", wav_bytes(8000, 800, 0.5), 8000)
    }

    #[test]
    fn follows_the_documented_state_machine() {
        let mut player = wav_player();
        assert_eq!(player.state, STATE_UNREALIZED);

        player.realize();
        assert_eq!(player.state, STATE_REALIZED);

        player.prefetch();
        assert_eq!(player.state, STATE_PREFETCHED);

        let mut audio = AudioRingBuffer::new(8000);
        player.start(&mut audio);
        assert_eq!(player.state, STATE_STARTED);

        player.stop();
        assert_eq!(player.state, STATE_PREFETCHED);

        player.close();
        assert_eq!(player.state, STATE_CLOSED);
    }

    #[test]
    fn start_does_not_emit_samples_twice() {
        let mut player = wav_player();
        let mut audio = AudioRingBuffer::new(8000);

        player.start(&mut audio);
        assert_eq!(
            audio.len(),
            0,
            "start must only arm playback; rendering writes samples"
        );

        player.render_audio_into_buffer(100, &mut audio);
        assert_eq!(audio.len(), 100);
    }

    #[test]
    fn stopped_player_renders_nothing() {
        let mut player = wav_player();
        let mut audio = AudioRingBuffer::new(8000);
        player.start(&mut audio);
        player.stop();

        player.render_audio_into_buffer(50, &mut audio);
        assert_eq!(audio.len(), 0);
    }

    #[test]
    fn restarting_rewinds_the_wav_play_head() {
        let mut player = wav_player();
        let mut audio = AudioRingBuffer::new(8000);

        player.start(&mut audio);
        player.render_audio_into_buffer(100, &mut audio);
        assert_eq!(player.wav_play_head, 100);

        player.stop();
        player.start(&mut audio);
        assert_eq!(player.wav_play_head, 0, "restart must rewind to the start");
    }

    #[test]
    fn volume_scales_rendered_wav() {
        let peak_with = |volume: i32| {
            let mut player = wav_player();
            player.set_volume(volume);
            let mut audio = AudioRingBuffer::new(8000);
            player.start(&mut audio);
            player.render_audio_into_buffer(400, &mut audio);
            let mut out = vec![0.0f32; 400];
            let n = audio.read_samples(&mut out);
            out[..n].iter().fold(0.0f32, |acc, s| acc.max(s.abs()))
        };

        let full = peak_with(100);
        let half = peak_with(50);
        let muted = peak_with(0);

        assert!(full > 0.1, "expected audible output, peak {full}");
        assert!(half < full, "50% must be quieter than 100%");
        assert_eq!(muted, 0.0, "0% must be silent");
    }

    #[test]
    fn loop_count_two_restarts_the_wav_once() {
        let mut player = Player::new(1, "audio/x-wav", wav_bytes(8000, 100, 0.5), 8000);
        player.set_loop_count(2);
        let mut audio = AudioRingBuffer::new(8000);
        player.start(&mut audio);

        // Render past the end of the clip.
        player.render_audio_into_buffer(150, &mut audio);
        assert_eq!(player.loop_count, 1, "one repetition must be consumed");
        assert!(player.wav_play_head < 100, "play head must restart");
        assert_eq!(player.state, STATE_STARTED);
    }

    #[test]
    fn non_looping_wav_returns_to_prefetched_at_the_end() {
        let mut player = Player::new(1, "audio/x-wav", wav_bytes(8000, 100, 0.5), 8000);
        player.set_loop_count(1);
        let mut audio = AudioRingBuffer::new(8000);
        player.start(&mut audio);
        player.render_audio_into_buffer(150, &mut audio);

        assert_eq!(player.state, STATE_PREFETCHED);
    }

    #[test]
    fn media_time_tracks_position_and_seeks_are_clamped() {
        let mut player = wav_player();
        let mut audio = AudioRingBuffer::new(8000);
        player.start(&mut audio);

        assert_eq!(player.media_time(), 0);
        player.render_audio_into_buffer(400, &mut audio); // 400 samples @ 8000 Hz = 50 ms
        assert!((player.media_time() - 50).abs() <= 1);

        // Seek forward within the clip.
        let landed = player.set_media_time(50);
        assert!((landed - 50).abs() <= 1, "landed at {landed}");
        assert_eq!(player.wav_play_head, 400);

        // Seek past the end clamps to the clip length.
        let landed = player.set_media_time(10_000);
        assert_eq!(landed, 100, "100 ms is the clip length at 800 samples");
        assert_eq!(player.wav_play_head, 800);
    }

    #[test]
    fn midi_player_renders_and_seeks() {
        let mut player = Player::new(2, "audio/midi", midi_bytes(), 44100);
        let mut audio = AudioRingBuffer::new(44100);
        player.start(&mut audio);
        player.render_audio_into_buffer(4410, &mut audio);

        let mut out = vec![0.0f32; 4410];
        let n = audio.read_samples(&mut out);
        assert_eq!(n, 4410);
        let audible = out.iter().filter(|s| s.abs() > 0.001).count();
        assert!(audible > 0, "MIDI player must produce audible samples");

        let landed = player.set_media_time(100);
        assert!(landed > 0, "MIDI seek must advance the position");
        assert_eq!(player.media_time(), landed);
    }

    #[test]
    fn ring_buffer_drops_oldest_samples_when_full() {
        let mut audio = AudioRingBuffer::new(100);
        for i in 0..300 {
            audio.write_sample(i as f32);
        }
        assert_eq!(audio.len(), 200, "capacity is two seconds of audio");
    }

    #[test]
    fn volume_control_clamp_and_mute() {
        let mut player = wav_player();
        player.set_volume(-50); // Underflow clamp
        assert_eq!(player.volume, 0);

        player.set_volume(150); // Overflow clamp
        assert_eq!(player.volume, 100);

        player.set_volume(0); // Mute
        assert_eq!(player.volume, 0);
    }

    #[test]
    fn sample_balance_and_underrun_overflow_metrics() {
        let mut audio = AudioRingBuffer::new(100); // max_capacity = 200
        for i in 0..250 {
            audio.write_sample((i as f32) * 0.01);
        }
        assert_eq!(audio.generated_samples, 250);
        assert_eq!(audio.overflow_count, 50);
        assert_eq!(audio.dropped_samples, 50);
        assert_eq!(audio.len(), 200);

        let mut out = vec![0.0f32; 250];
        let read = audio.read_samples(&mut out);
        assert_eq!(read, 200);
        assert_eq!(audio.consumed_samples, 200);
        assert_eq!(audio.underrun_count, 1);
        assert_eq!(audio.len(), 0);

        // Verification of sample balance: generated = consumed + dropped + remaining
        assert_eq!(
            audio.generated_samples,
            audio.consumed_samples + audio.dropped_samples + audio.len() as u64
        );

        let (peak, rms, dc_offset) = AudioRingBuffer::compute_metrics(&out[..read]);
        assert!(peak > 0.0);
        assert!(rms > 0.0);
        assert!(dc_offset > 0.0);
    }

    #[test]
    fn wav_unsupported_32_bit_rejected() {
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&52u32.to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // 1 channel
        wav.extend_from_slice(&8000u32.to_le_bytes());
        wav.extend_from_slice(&32000u32.to_le_bytes());
        wav.extend_from_slice(&4u16.to_le_bytes());
        wav.extend_from_slice(&32u16.to_le_bytes()); // 32 bits per sample
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&[0u8; 16]);

        let err = WavDecoder::parse(&wav, 8000).expect_err("32-bit PCM must be rejected");
        assert!(err.contains("unsupported PCM bit depth"));
    }

    #[test]
    fn test_dc_blocking_filter_eliminates_dc_bias_under_10k_samples() {
        let mut audio = AudioRingBuffer::new(44100);
        // Simulate continuous playback with severe DC offset bias (+0.35 offset + audio wave)
        // 441 Hz at 44100 Hz has an integer period of 100 samples.
        for i in 0..10_000 {
            let t = i as f32 / 44100.0;
            let raw = 0.35 + 0.40 * (2.0 * std::f32::consts::PI * 441.0 * t).sin();
            audio.write_sample(raw);
        }

        // Read out the first 4,000 samples to pass through the initial transient
        let mut transient = vec![0.0f32; 4000];
        let _ = audio.read_samples(&mut transient);

        // Read remaining steady-state samples generated during the 10,000-sample playback window
        let mut steady_state = vec![0.0f32; 5000];
        let n = audio.read_samples(&mut steady_state);
        assert!(n >= 5000, "Expected at least 5000 steady-state samples");

        // Verify arithmetic mean of steady-state audio strictly tends to 0.0 (<= 0.0001)
        let (_, _, dc_bias) = AudioRingBuffer::compute_metrics(&steady_state[..n]);
        assert!(
            dc_bias.abs() <= 0.0001,
            "DC bias after continuous playback must be <= 0.0001, got: {}",
            dc_bias
        );

        // Also test constant DC bias step response after 10,000 continuous samples
        let mut audio2 = AudioRingBuffer::new(44100);
        for _ in 0..10_000 {
            audio2.write_sample(0.5);
        }
        // Drain initial buffer
        let mut drain = vec![0.0f32; 10_000];
        let _ = audio2.read_samples(&mut drain);

        // After 10,000 samples, write 2,000 additional samples with the same 0.5 DC bias
        for _ in 0..2_000 {
            audio2.write_sample(0.5);
        }
        let mut test_buf = vec![0.0f32; 2_000];
        let count = audio2.read_samples(&mut test_buf);
        let (_, _, dc_bias2) = AudioRingBuffer::compute_metrics(&test_buf[..count]);
        assert!(
            dc_bias2.abs() <= 0.0001,
            "Steady-state DC bias after 10k samples must be <= 0.0001, got: {}",
            dc_bias2
        );
    }
}
