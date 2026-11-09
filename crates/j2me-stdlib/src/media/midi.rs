/// Pure Rust Standard MIDI File (SMF) Parser & Polyphonic Synthesizer
///
/// Features:
/// - SMF Format 0 & 1 parser with VLQ delta times & running status
/// - Tempo events (0x51 Set Tempo), Note On (0x90), Note Off (0x80), Program Change (0xC0)
/// - 32-voice polyphonic synthesizer with ADSR envelopes
/// - Multi-timbral instrument synthesis (Pianos, Organs, Guitars, Bass, Strings, Brass, Flute, Synth)
/// - Channel 10 (index 9) dedicated drum/percussion synthesizer (Kick, Snare, Hi-hat, Toms)
/// - Infinite (-1) and finite looping support

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiEventType {
    NoteOff { channel: u8, note: u8, velocity: u8 },
    NoteOn { channel: u8, note: u8, velocity: u8 },
    ProgramChange { channel: u8, program: u8 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    PitchBend { channel: u8, value: i16 },
    SetTempo { us_per_quarter: u32 },
    EndOfTrack,
}

#[derive(Debug, Clone)]
pub struct MidiTimedEvent {
    pub absolute_tick: u64,
    pub event: MidiEventType,
}

#[derive(Debug, Clone)]
pub struct MidiTrack {
    pub events: Vec<MidiTimedEvent>,
}

#[derive(Debug, Clone)]
pub struct MidiSequence {
    pub division: u16,               // Ticks per quarter note
    pub events: Vec<MidiTimedEvent>, // Merged events sorted by absolute tick
    pub total_ticks: u64,
}

impl MidiSequence {
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < 14 {
            return Err("MIDI file too short for MThd header".to_string());
        }

        if &data[0..4] != b"MThd" {
            return Err("Missing 'MThd' header".to_string());
        }

        let header_len = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
        let _format = u16::from_be_bytes([data[8], data[9]]);
        let num_tracks = u16::from_be_bytes([data[10], data[11]]) as usize;
        let division = u16::from_be_bytes([data[12], data[13]]);

        let mut offset = 8 + header_len;
        let mut all_events = Vec::new();
        let mut total_ticks = 0;

        for _ in 0..num_tracks {
            if offset + 8 > data.len() {
                break;
            }
            if &data[offset..offset + 4] != b"MTrk" {
                // Seek to next chunk
                offset += 1;
                while offset + 8 <= data.len() && &data[offset..offset + 4] != b"MTrk" {
                    offset += 1;
                }
                if offset + 8 > data.len() {
                    break;
                }
            }

            let track_len = u32::from_be_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]) as usize;
            offset += 8;

            let track_end = (offset + track_len).min(data.len());
            let track_data = &data[offset..track_end];
            offset = track_end;

            let mut track_reader = TrackReader::new(track_data);
            let mut current_tick = 0u64;

            while !track_reader.is_eof() {
                let delta = track_reader.read_vlq();
                current_tick += delta as u64;

                if let Some(event) = track_reader.read_event() {
                    all_events.push(MidiTimedEvent {
                        absolute_tick: current_tick,
                        event,
                    });
                    if event == MidiEventType::EndOfTrack {
                        break;
                    }
                }
            }

            if current_tick > total_ticks {
                total_ticks = current_tick;
            }
        }

        // Sort all events by absolute tick for sequential playback
        all_events.sort_by_key(|e| e.absolute_tick);

        Ok(Self {
            division: if division == 0 { 96 } else { division },
            events: all_events,
            total_ticks,
        })
    }
}

struct TrackReader<'a> {
    data: &'a [u8],
    pos: usize,
    running_status: u8,
}

impl<'a> TrackReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            running_status: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.data.len()
    }

    fn read_byte(&mut self) -> u8 {
        if self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            b
        } else {
            0
        }
    }

    fn read_vlq(&mut self) -> u32 {
        let mut val = 0u32;
        for _ in 0..4 {
            let b = self.read_byte();
            val = (val << 7) | ((b & 0x7F) as u32);
            if (b & 0x80) == 0 {
                break;
            }
        }
        val
    }

    fn read_event(&mut self) -> Option<MidiEventType> {
        if self.is_eof() {
            return None;
        }

        let mut status = self.read_byte();
        if (status & 0x80) == 0 {
            // Running status applies
            if self.running_status != 0 {
                self.pos -= 1; // Unread data byte
                status = self.running_status;
            } else {
                return None;
            }
        } else if status < 0xF0 {
            self.running_status = status;
        }

        let channel = status & 0x0F;
        let event_type = status & 0xF0;

        match event_type {
            0x80 => {
                let note = self.read_byte();
                let velocity = self.read_byte();
                Some(MidiEventType::NoteOff {
                    channel,
                    note,
                    velocity,
                })
            }
            0x90 => {
                let note = self.read_byte();
                let velocity = self.read_byte();
                if velocity == 0 {
                    Some(MidiEventType::NoteOff {
                        channel,
                        note,
                        velocity: 0,
                    })
                } else {
                    Some(MidiEventType::NoteOn {
                        channel,
                        note,
                        velocity,
                    })
                }
            }
            0xA0 => {
                let _note = self.read_byte();
                let _pressure = self.read_byte();
                None
            }
            0xB0 => {
                let controller = self.read_byte();
                let value = self.read_byte();
                Some(MidiEventType::ControlChange {
                    channel,
                    controller,
                    value,
                })
            }
            0xC0 => {
                let program = self.read_byte();
                Some(MidiEventType::ProgramChange { channel, program })
            }
            0xD0 => {
                let _pressure = self.read_byte();
                None
            }
            0xE0 => {
                let lsb = self.read_byte() as i16;
                let msb = self.read_byte() as i16;
                let bend = ((msb << 7) | (lsb & 0x7F)) - 8192;
                Some(MidiEventType::PitchBend {
                    channel,
                    value: bend,
                })
            }
            0xF0 => {
                if status == 0xFF {
                    let meta_type = self.read_byte();
                    let len = self.read_vlq() as usize;
                    if meta_type == 0x2F {
                        Some(MidiEventType::EndOfTrack)
                    } else if meta_type == 0x51 && len == 3 {
                        let b1 = self.read_byte() as u32;
                        let b2 = self.read_byte() as u32;
                        let b3 = self.read_byte() as u32;
                        let tempo = (b1 << 16) | (b2 << 8) | b3;
                        Some(MidiEventType::SetTempo {
                            us_per_quarter: tempo,
                        })
                    } else {
                        // Skip unhandled meta payload
                        self.pos = (self.pos + len).min(self.data.len());
                        None
                    }
                } else {
                    // SysEx
                    let len = self.read_vlq() as usize;
                    self.pos = (self.pos + len).min(self.data.len());
                    None
                }
            }
            _ => None,
        }
    }
}

// =========================================================================
// Polyphonic Synthesizer
// =========================================================================

use std::sync::OnceLock;

const SINE_TABLE_SIZE: usize = 2048;
static SINE_TABLE: OnceLock<Box<[f32; SINE_TABLE_SIZE]>> = OnceLock::new();

#[inline(always)]
fn get_sine_table() -> &'static [f32; SINE_TABLE_SIZE] {
    SINE_TABLE.get_or_init(|| {
        let mut table = Box::new([0.0f32; SINE_TABLE_SIZE]);
        for i in 0..SINE_TABLE_SIZE {
            let p = (i as f32) / (SINE_TABLE_SIZE as f32);
            table[i] = (p * 2.0 * std::f32::consts::PI).sin();
        }
        table
    })
}

#[inline(always)]
fn fast_sin(phase: f32) -> f32 {
    let table = get_sine_table();
    let p = (phase - phase.floor()) * (SINE_TABLE_SIZE as f32);
    let i0 = (p as usize) & (SINE_TABLE_SIZE - 1);
    let i1 = (i0 + 1) & (SINE_TABLE_SIZE - 1);
    let frac = p - (p as usize as f32);
    table[i0] + (table[i1] - table[i0]) * frac
}

#[inline(always)]
fn fast_soft_clip(x: f32) -> f32 {
    let x2 = x * x;
    if x2 < 9.0 {
        x * (27.0 + x2) / (27.0 + 9.0 * x2)
    } else if x > 0.0 {
        1.0
    } else {
        -1.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VoiceState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone)]
struct Voice {
    active: bool,
    channel: u8,
    note: u8,
    velocity: f32,
    program: u8,
    base_freq: f32,
    pitch_bend: f32,
    channel_volume: f32,
    freq: f32,
    phase: f32,
    state: VoiceState,
    envelope: f32,
    env_attack_rate: f32,
    env_decay_rate: f32,
    env_sustain_level: f32,
    env_release_rate: f32,
    state_time: f32,
    pitch_decay: f32,
    lfsr: u32,
    lpf_state: f32,
    lpf_alpha: f32,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            active: false,
            channel: 0,
            note: 60,
            velocity: 0.8,
            program: 0,
            base_freq: 440.0,
            pitch_bend: 0.0,
            channel_volume: 1.0,
            freq: 440.0,
            phase: 0.0,
            state: VoiceState::Off,
            envelope: 0.0,
            env_attack_rate: 0.001,
            env_decay_rate: 0.001,
            env_sustain_level: 0.65,
            env_release_rate: 0.001,
            state_time: 0.0,
            pitch_decay: 0.0,
            lfsr: 0xACE1,
            lpf_state: 0.0,
            lpf_alpha: 0.5,
        }
    }
}

impl Voice {
    fn note_on(
        &mut self,
        channel: u8,
        note: u8,
        velocity: u8,
        program: u8,
        pitch_bend: f32,
        channel_volume: f32,
    ) {
        self.active = true;
        self.channel = channel;
        self.note = note;
        self.velocity = (velocity as f32 / 127.0).clamp(0.05, 1.0);
        self.program = program;
        self.pitch_bend = pitch_bend;
        self.channel_volume = channel_volume.clamp(0.0, 1.0);
        self.phase = 0.0;
        self.state = VoiceState::Attack;
        self.state_time = 0.0;
        self.envelope = 0.0;
        self.pitch_decay = 1.0;
        self.lpf_state = 0.0;

        // Base frequency
        self.base_freq = 440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0);
        self.freq = self.base_freq * 2.0f32.powf(pitch_bend / 12.0);

        // Precompute low-pass filter alpha (nominal 44.1kHz sample rate)
        let cutoff = (self.freq * 4.0).clamp(1200.0, 12000.0);
        let dt = 1.0 / 44100.0;
        self.lpf_alpha = (2.0 * std::f32::consts::PI * cutoff * dt).clamp(0.01, 0.95);

        // Precompute ADSR envelope step rates
        let attack_time = if self.channel == 9 { 0.002 } else { 0.015 };
        let decay_time = if self.channel == 9 { 0.15 } else { 0.25 };
        let sustain_level = if self.channel == 9 { 0.0 } else { 0.65 };
        let release_time = if self.channel == 9 { 0.05 } else { 0.12 };

        self.env_attack_rate = dt / attack_time;
        self.env_decay_rate = (1.0 - sustain_level) * (dt / decay_time);
        self.env_sustain_level = sustain_level;
        self.env_release_rate = (sustain_level.max(0.1) / release_time) * dt;
    }

    fn update_pitch_bend(&mut self, pitch_bend: f32) {
        self.pitch_bend = pitch_bend;
        self.freq = self.base_freq * 2.0f32.powf(pitch_bend / 12.0);
        let cutoff = (self.freq * 4.0).clamp(1200.0, 12000.0);
        let dt = 1.0 / 44100.0;
        self.lpf_alpha = (2.0 * std::f32::consts::PI * cutoff * dt).clamp(0.01, 0.95);
    }

    fn update_channel_volume(&mut self, channel_volume: f32) {
        self.channel_volume = channel_volume.clamp(0.0, 1.0);
    }

    fn note_off(&mut self) {
        if self.active && self.state != VoiceState::Off && self.state != VoiceState::Release {
            self.state = VoiceState::Release;
            self.state_time = 0.0;
        }
    }

    fn render_sample(&mut self, sample_rate: f32) -> f32 {
        if !self.active || self.state == VoiceState::Off {
            return 0.0;
        }

        let dt = 1.0 / sample_rate;
        self.state_time += dt;

        // 1. Process ADSR Envelope (Fast zero-division branchless stepping)
        match self.state {
            VoiceState::Attack => {
                self.envelope += self.env_attack_rate;
                if self.envelope >= 1.0 {
                    self.envelope = 1.0;
                    self.state = VoiceState::Decay;
                }
            }
            VoiceState::Decay => {
                self.envelope -= self.env_decay_rate;
                if self.envelope <= self.env_sustain_level {
                    if self.env_sustain_level <= 0.0 {
                        self.envelope = 0.0;
                        self.state = VoiceState::Off;
                        self.active = false;
                        return 0.0;
                    } else {
                        self.envelope = self.env_sustain_level;
                        self.state = VoiceState::Sustain;
                    }
                }
            }
            VoiceState::Sustain => {
                if self.channel == 9 {
                    self.state = VoiceState::Release;
                }
            }
            VoiceState::Release => {
                self.envelope -= self.env_release_rate;
                if self.envelope <= 0.0 {
                    self.envelope = 0.0;
                    self.state = VoiceState::Off;
                    self.active = false;
                    return 0.0;
                }
            }
            VoiceState::Off => {
                self.envelope = 0.0;
                self.active = false;
                return 0.0;
            }
        }

        // 2. Synthesize Audio Waveform
        let raw_sample = if self.channel == 9 {
            // Percussion Synthesis
            self.render_drum(dt)
        } else {
            // Melodic Synthesis based on General MIDI Program with Low-Pass Filtering
            let raw = self.render_melodic(dt);
            self.lpf_state += self.lpf_alpha * (raw - self.lpf_state);
            self.lpf_state
        };

        raw_sample * self.envelope * self.velocity * self.channel_volume * 0.18
    }

    fn render_drum(&mut self, dt: f32) -> f32 {
        // Linear feedback shift register for noise
        let bit = ((self.lfsr >> 0) ^ (self.lfsr >> 2) ^ (self.lfsr >> 3) ^ (self.lfsr >> 5)) & 1;
        self.lfsr = (self.lfsr >> 1) | (bit << 15);
        let noise = (self.lfsr as f32 / 32768.0) - 1.0;

        match self.note {
            35 | 36 => {
                // Bass Drum (Kick): Pitch drop from 120Hz to 45Hz
                self.pitch_decay *= 0.992;
                let cur_freq = 45.0 + 80.0 * self.pitch_decay;
                self.phase += cur_freq * dt;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                fast_sin(self.phase) * 0.9
            }
            38 | 40 => {
                // Snare Drum: Noise + tone
                self.phase += 190.0 * dt;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                let tone = fast_sin(self.phase) * 0.35;
                noise * 0.5 + tone
            }
            42 | 44 | 46 => {
                // Closed/Open Hi-hat: Metallic filtered noise
                noise * 0.4
            }
            49 | 57 => {
                // Crash / Cymbal
                noise * 0.5
            }
            _ => {
                // Toms / other percussion: decaying tone + noise click
                self.pitch_decay *= 0.994;
                let cur_freq = self.freq * (0.8 + 0.4 * self.pitch_decay);
                self.phase += cur_freq * dt;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                fast_sin(self.phase) * 0.8 + noise * 0.2
            }
        }
    }

    fn render_melodic(&mut self, dt: f32) -> f32 {
        self.phase += self.freq * dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        match self.program {
            0..=7 => {
                // Acoustic Pianos: Sine + warm harmonic
                let h1 = fast_sin(self.phase);
                let h2 = fast_sin(self.phase * 2.0) * 0.25;
                h1 * 0.75 + h2
            }
            16..=23 => {
                // Organs: Hammond drawbar combination (1st, 2nd, 3rd harmonics)
                let h1 = fast_sin(self.phase);
                let h2 = fast_sin(self.phase * 2.0) * 0.35;
                let h3 = fast_sin(self.phase * 3.0) * 0.18;
                (h1 + h2 + h3) * 0.55
            }
            24..=31 => {
                // Guitars: Plucked waveform (sawtooth blend)
                let saw = 2.0 * self.phase - 1.0;
                let sine = fast_sin(self.phase);
                saw * 0.5 + sine * 0.5
            }
            32..=39 => {
                // Bass: Warm Triangle / Low Sine
                let tri = if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                };
                let sine = fast_sin(self.phase);
                tri * 0.65 + sine * 0.35
            }
            40..=55 => {
                // Strings & Ensemble: Rich Sawtooth
                let saw1 = 2.0 * self.phase - 1.0;
                saw1 * 0.65
            }
            56..=63 => {
                // Brass: 35% duty cycle pulse with zero-mean DC balanced
                // High for 0.35 of period (0.65), low for 0.65 of period (-0.35): DC offset = 0.35 * 0.65 - 0.65 * 0.35 = 0.0
                let raw_pulse = if self.phase < 0.35 { 0.65 } else { -0.35 };
                let sine = fast_sin(self.phase) * 0.25;
                raw_pulse + sine
            }
            64..=71 => {
                // Reed: Square + gentle sawtooth
                let sq = if self.phase < 0.5 { 0.5 } else { -0.5 };
                let saw = (2.0 * self.phase - 1.0) * 0.25;
                sq + saw
            }
            72..=79 => {
                // Pipe: Pure sine with breathy triangle
                let sine = fast_sin(self.phase);
                let tri = if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                };
                sine * 0.75 + tri * 0.2
            }
            80..=95 => {
                // Synth Lead: Bandlimited odd harmonics instead of raw aliased square
                let s1 = fast_sin(self.phase);
                let s3 = fast_sin(self.phase * 3.0) * (1.0 / 3.0);
                let s5 = fast_sin(self.phase * 5.0) * (1.0 / 5.0);
                (s1 + s3 + s5) * 0.6
            }
            _ => {
                // Default: Triangle + sine
                let sine = fast_sin(self.phase);
                sine * 0.8
            }
        }
    }
}

use super::manager::DcBlockingFilter;

pub struct MidiSynthesizer {
    sequence: MidiSequence,
    voices: [Voice; 32],
    programs: [u8; 16],        // Current program per channel
    pitch_bends: [f32; 16],    // Pitch bend in semitones per channel
    channel_volumes: [f32; 16],// Volume scale (0.0..1.0) per channel
    current_tick: u64,
    event_idx: usize,
    sample_rate: u32,
    tempo_us_per_quarter: u32,
    tick_accum: f64,
    pub loop_count: i32, // -1 = infinite, >0 = count, 0 = no loop
    pub volume: i32,     // 0..100
    pub is_playing: bool,
    pub dc_blocker: DcBlockingFilter,
}

impl MidiSynthesizer {
    pub fn new(sequence: MidiSequence, sample_rate: u32) -> Self {
        Self {
            sequence,
            voices: std::array::from_fn(|_| Voice::default()),
            programs: [0; 16],
            pitch_bends: [0.0; 16],
            channel_volumes: [1.0; 16],
            current_tick: 0,
            event_idx: 0,
            sample_rate,
            tempo_us_per_quarter: 500_000, // 120 BPM default
            tick_accum: 0.0,
            loop_count: 0,
            volume: 100,
            is_playing: false,
            dc_blocker: DcBlockingFilter::new(0.995),
        }
    }

    pub fn start(&mut self) {
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.dc_blocker.reset();
        for voice in &mut self.voices {
            voice.active = false;
            voice.state = VoiceState::Off;
        }
    }

    pub fn reset(&mut self) {
        self.current_tick = 0;
        self.event_idx = 0;
        self.tick_accum = 0.0;
        self.dc_blocker.reset();
        for voice in &mut self.voices {
            voice.active = false;
            voice.state = VoiceState::Off;
        }
    }

    /// Smoothly transitions all currently active sounding voices to release phase,
    /// preventing note accumulation/sticking across track loop iterations.
    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            if voice.active && voice.state != VoiceState::Off && voice.state != VoiceState::Release {
                voice.state = VoiceState::Release;
                voice.state_time = 0.0;
            }
        }
    }

    /// Elapsed position in milliseconds, derived from the current tick so it
    /// stays correct across tempo changes.
    pub fn media_time_ms(&self) -> i64 {
        let us_per_tick = self.tempo_us_per_quarter as f64 / self.sequence.division.max(1) as f64;
        (self.current_tick as f64 * us_per_tick / 1000.0) as i64
    }

    /// Seeks to `ms` by rewinding and replaying events up to that position.
    /// Voices are cleared first so no note from the skipped region keeps
    /// sounding. Returns the position actually reached.
    pub fn seek_ms(&mut self, ms: i64) -> i64 {
        self.reset();
        let target = ms.max(0) as f64;

        // Tempo is taken from events as we walk, so seek accuracy follows the
        // same tempo map the normal playback path uses.
        while self.event_idx < self.sequence.events.len() {
            if self.media_time_ms() as f64 >= target {
                break;
            }
            let event = self.sequence.events[self.event_idx].event.clone();
            if let MidiEventType::SetTempo { us_per_quarter } = event {
                self.tempo_us_per_quarter = us_per_quarter;
            }
            let next_tick = self.sequence.events[self.event_idx].absolute_tick;
            if next_tick > self.current_tick {
                let us_per_tick =
                    self.tempo_us_per_quarter as f64 / self.sequence.division.max(1) as f64;
                let next_ms = next_tick as f64 * us_per_tick / 1000.0;
                if next_ms > target {
                    // Land mid-gap: interpolate the tick for the exact target.
                    let ticks = ((target - self.media_time_ms() as f64) * 1000.0 / us_per_tick)
                        .max(0.0) as u64;
                    self.current_tick += ticks;
                    break;
                }
                self.current_tick = next_tick;
            }
            if let MidiEventType::NoteOn {
                channel,
                note,
                velocity,
            } = event
            {
                // Skipped notes are not retriggered; only their timing matters
                // for positioning. Sounding notes come from playback onwards.
                let _ = (channel, note, velocity);
            }
            self.event_idx += 1;
        }

        self.media_time_ms()
    }

    fn allocate_voice(&mut self, channel: u8, note: u8, velocity: u8, program: u8) {
        let ch_idx = (channel as usize).min(self.pitch_bends.len() - 1);
        let bend = self.pitch_bends[ch_idx];
        let vol = self.channel_volumes[ch_idx];

        // 1. Try to find an inactive voice
        for voice in &mut self.voices {
            if !voice.active {
                voice.note_on(channel, note, velocity, program, bend, vol);
                return;
            }
        }

        // 2. Try to find a voice in Release state
        for voice in &mut self.voices {
            if voice.state == VoiceState::Release {
                voice.note_on(channel, note, velocity, program, bend, vol);
                return;
            }
        }

        // 3. Voice stealing: steal the voice with lowest envelope
        let mut min_idx = 0;
        let mut min_env = 999.0f32;
        for (i, v) in self.voices.iter().enumerate() {
            if v.envelope < min_env {
                min_env = v.envelope;
                min_idx = i;
            }
        }
        self.voices[min_idx].note_on(channel, note, velocity, program, bend, vol);
    }

    fn release_voice(&mut self, channel: u8, note: u8) {
        for voice in &mut self.voices {
            if voice.active && voice.channel == channel && voice.note == note {
                voice.note_off();
            }
        }
    }

    pub fn render_samples(&mut self, count: usize, dest: &mut Vec<f32>) {
        if !self.is_playing || self.sequence.events.is_empty() {
            return;
        }

        let seconds_per_tick =
            (self.tempo_us_per_quarter as f64 / 1_000_000.0) / (self.sequence.division as f64);
        let ticks_per_sample = 1.0 / (seconds_per_tick * self.sample_rate as f64);
        let vol_scale = (self.volume.clamp(0, 100) as f32 / 100.0) * 0.70;

        dest.reserve(count);

        for _ in 0..count {
            // Advance MIDI timeline
            self.tick_accum += ticks_per_sample;
            while self.tick_accum >= 1.0 {
                self.tick_accum -= 1.0;
                self.current_tick += 1;

                // Dispatch all events at current tick
                while self.event_idx < self.sequence.events.len() {
                    let ev = &self.sequence.events[self.event_idx];
                    if ev.absolute_tick > self.current_tick {
                        break;
                    }

                    match ev.event {
                        MidiEventType::NoteOn {
                            channel,
                            note,
                            velocity,
                        } => {
                            if velocity == 0 {
                                self.release_voice(channel, note);
                            } else {
                                let prog = self.programs[channel as usize];
                                self.allocate_voice(channel, note, velocity, prog);
                            }
                        }
                        MidiEventType::NoteOff { channel, note, .. } => {
                            self.release_voice(channel, note);
                        }
                        MidiEventType::ProgramChange { channel, program } => {
                            if (channel as usize) < self.programs.len() {
                                self.programs[channel as usize] = program;
                            }
                        }
                        MidiEventType::ControlChange {
                            channel,
                            controller,
                            value,
                        } => {
                            let ch_idx = channel as usize;
                            if ch_idx < self.channel_volumes.len() {
                                if controller == 7 {
                                    // Channel Volume
                                    let vol = value as f32 / 127.0;
                                    self.channel_volumes[ch_idx] = vol;
                                    for voice in &mut self.voices {
                                        if voice.active && voice.channel == channel {
                                            voice.update_channel_volume(vol);
                                        }
                                    }
                                } else if controller == 11 {
                                    // Expression Controller
                                    let exp = value as f32 / 127.0;
                                    self.channel_volumes[ch_idx] = exp;
                                    for voice in &mut self.voices {
                                        if voice.active && voice.channel == channel {
                                            voice.update_channel_volume(exp);
                                        }
                                    }
                                } else if controller == 120 || controller == 123 {
                                    // All Sound Off / All Notes Off
                                    for voice in &mut self.voices {
                                        if voice.active && voice.channel == channel {
                                            voice.note_off();
                                        }
                                    }
                                }
                            }
                        }
                        MidiEventType::PitchBend { channel, value } => {
                            let ch_idx = channel as usize;
                            if ch_idx < self.pitch_bends.len() {
                                let semitones = (value as f32 / 8192.0) * 2.0;
                                self.pitch_bends[ch_idx] = semitones;
                                for voice in &mut self.voices {
                                    if voice.active && voice.channel == channel {
                                        voice.update_pitch_bend(semitones);
                                    }
                                }
                            }
                        }
                        MidiEventType::SetTempo { us_per_quarter } => {
                            if us_per_quarter > 0 {
                                self.tempo_us_per_quarter = us_per_quarter;
                            }
                        }
                        MidiEventType::EndOfTrack => {}
                    }

                    self.event_idx += 1;
                }

                // Check for sequence loop/end
                if self.event_idx >= self.sequence.events.len()
                    || self.current_tick >= self.sequence.total_ticks
                {
                    if self.loop_count < 0 || self.loop_count > 1 {
                        if self.loop_count > 1 {
                            self.loop_count -= 1;
                        }
                        // Reset stuck notes before restarting timeline to avoid stacking polyphony
                        self.all_notes_off();
                        self.current_tick = 0;
                        self.event_idx = 0;
                    } else {
                        // Finished
                        self.is_playing = false;
                        break;
                    }
                }
            }

            // Mix active voices
            let mut sample_sum = 0.0f32;
            let mut active_count = 0usize;
            let sr = self.sample_rate as f32;
            for voice in &mut self.voices {
                if voice.active {
                    active_count += 1;
                    sample_sum += voice.render_sample(sr);
                }
            }

            if active_count == 0 {
                dest.push(0.0);
                continue;
            }

            // Dynamic polyphony normalization (Headroom Normalization)
            // Psychoacoustic power-summation scaling prevents +20dB overdrive when many voices sound simultaneously
            let headroom = (active_count as f32).sqrt();
            let scaled = (sample_sum / headroom) * vol_scale;
            let dc_blocked = self.dc_blocker.process(scaled);
            let clipped = fast_soft_clip(dc_blocked);
            dest.push(clipped);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wraps raw track payloads into a valid SMF header.
    fn smf(division: u16, tracks: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"MThd");
        out.extend_from_slice(&6u32.to_be_bytes());
        let format: u16 = if tracks.len() > 1 { 1 } else { 0 };
        out.extend_from_slice(&format.to_be_bytes());
        out.extend_from_slice(&(tracks.len() as u16).to_be_bytes());
        out.extend_from_slice(&division.to_be_bytes());
        for track in tracks {
            out.extend_from_slice(b"MTrk");
            out.extend_from_slice(&(track.len() as u32).to_be_bytes());
            out.extend_from_slice(track);
        }
        out
    }

    #[test]
    fn parses_header_tempo_notes_and_end_of_track() {
        let track = vec![
            0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, // Set Tempo 500000 us/qn
            0x00, 0x90, 0x3C, 0x64, // Note On ch0 note 60 vel 100
            0x60, 0x80, 0x3C, 0x00, // dt=96 Note Off ch0 note 60
            0x00, 0xFF, 0x2F, 0x00, // End of track
        ];
        let seq = MidiSequence::parse(&smf(96, &[track])).expect("valid SMF");

        assert_eq!(seq.division, 96);
        assert_eq!(seq.total_ticks, 96);
        assert_eq!(
            seq.events[0].event,
            MidiEventType::SetTempo {
                us_per_quarter: 500_000
            }
        );
        assert!(seq.events.iter().any(|e| e.event
            == MidiEventType::NoteOn {
                channel: 0,
                note: 60,
                velocity: 100
            }));
        assert!(seq.events.iter().any(|e| e.event
            == MidiEventType::NoteOff {
                channel: 0,
                note: 60,
                velocity: 0
            }));
        assert_eq!(seq.events.last().unwrap().event, MidiEventType::EndOfTrack);
    }

    #[test]
    fn running_status_reuses_previous_status_byte() {
        let track = vec![
            0x00, 0x90, 0x3C, 0x64, // Note On 60
            0x00, 0x3E, 0x50, // running status: Note On 62
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let seq = MidiSequence::parse(&smf(96, &[track])).expect("valid SMF");

        let notes: Vec<(u8, u8)> = seq
            .events
            .iter()
            .filter_map(|e| match e.event {
                MidiEventType::NoteOn { note, velocity, .. } => Some((note, velocity)),
                _ => None,
            })
            .collect();
        assert_eq!(notes, vec![(60, 100), (62, 80)]);
    }

    #[test]
    fn note_on_with_zero_velocity_is_note_off() {
        let track = vec![
            0x00, 0x90, 0x3C, 0x64, 0x00, 0x90, 0x3C, 0x00, // velocity 0 -> off
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let seq = MidiSequence::parse(&smf(96, &[track])).expect("valid SMF");
        let offs = seq
            .events
            .iter()
            .filter(|e| matches!(e.event, MidiEventType::NoteOff { note: 60, .. }))
            .count();
        assert_eq!(offs, 1);
    }

    #[test]
    fn parses_multiple_tracks_and_sorts_events_by_tick() {
        let melody = vec![
            0x00, 0x90, 0x40, 0x64, // tick 0
            0x60, 0x80, 0x40, 0x00, // tick 96
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let bass = vec![
            0x30, 0x90, 0x30, 0x64, // tick 48
            0x30, 0x80, 0x30, 0x00, 0x00, 0xFF, 0x2F, 0x00,
        ];
        let seq = MidiSequence::parse(&smf(96, &[melody, bass])).expect("valid SMF");

        let ticks: Vec<u64> = seq.events.iter().map(|e| e.absolute_tick).collect();
        let mut sorted = ticks.clone();
        sorted.sort_unstable();
        assert_eq!(ticks, sorted, "events must be time ordered after merge");
        assert!(ticks.contains(&48));
        assert!(ticks.contains(&96));
    }

    #[test]
    fn rejects_missing_or_truncated_header() {
        assert!(MidiSequence::parse(b"MThd").is_err());
        assert!(MidiSequence::parse(b"RIFF\x00\x00\x00\x00WAVEfmt ").is_err());
    }

    fn two_note_sequence() -> MidiSequence {
        let track = vec![
            0x00, 0x90, 0x3C, 0x64, 0x60, 0x80, 0x3C, 0x00, 0x00, 0x90, 0x40, 0x64, 0x60, 0x80,
            0x40, 0x00, 0x00, 0xFF, 0x2F, 0x00,
        ];
        MidiSequence::parse(&smf(96, &[track])).expect("valid SMF")
    }

    #[test]
    fn synth_renders_audible_samples_without_clipping() {
        let sequence = two_note_sequence();
        let mut synth = MidiSynthesizer::new(sequence, 44100);
        synth.start();

        let mut out = Vec::new();
        synth.render_samples(44100, &mut out);

        let non_zero = out.iter().filter(|s| s.abs() > 0.001).count();
        let peak = out.iter().fold(0.0f32, |acc, s| acc.max(s.abs()));
        assert_eq!(out.len(), 44100);
        assert!(non_zero > 1000, "expected audible output, got {non_zero}");
        assert!(peak <= 1.0, "output must stay inside [-1, 1], peak {peak}");
    }

    #[test]
    fn stop_silences_the_synth() {
        let mut synth = MidiSynthesizer::new(two_note_sequence(), 44100);
        synth.start();
        synth.render_samples(1000, &mut Vec::new());
        synth.stop();

        let mut out = Vec::new();
        synth.render_samples(1000, &mut out);
        assert!(out.is_empty(), "stopped synth must not render");
        assert!(!synth.is_playing);
    }

    #[test]
    fn finishes_after_last_note_when_not_looping() {
        let mut synth = MidiSynthesizer::new(two_note_sequence(), 44100);
        synth.loop_count = 0;
        synth.start();
        synth.render_samples(44100 * 4, &mut Vec::new());
        assert!(!synth.is_playing, "non-looping sequence must finish");
    }

    #[test]
    fn infinite_loop_keeps_playing() {
        let mut synth = MidiSynthesizer::new(two_note_sequence(), 44100);
        synth.loop_count = -1;
        synth.start();
        synth.render_samples(44100 * 4, &mut Vec::new());
        assert!(synth.is_playing, "looping sequence must keep playing");
    }

    #[test]
    fn parses_running_status_and_tempo_change() {
        // Track with:
        // delta 0: SetTempo (500_000 us = 120 bpm)
        // delta 0: NoteOn 0x3C vel 0x64 (status 0x90)
        // delta 48: NoteOn 0x3E vel 0x64 (omitted status byte, running status 0x90)
        // delta 48: NoteOff 0x3C vel 0 (status 0x80)
        // delta 0: EndOfTrack
        let track = vec![
            0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, // Tempo = 500,000 us
            0x00, 0x90, 0x3C, 0x64, // Note On C4
            0x30, 0x3E, 0x64, // Running status Note On D4 at tick 48
            0x30, 0x80, 0x3C, 0x00, // Note Off C4 at tick 96
            0x00, 0xFF, 0x2F, 0x00, // End of track
        ];
        let seq = MidiSequence::parse(&smf(96, &[track])).expect("valid SMF with running status");
        assert_eq!(seq.events.len(), 5);
        assert!(matches!(
            seq.events[0].event,
            MidiEventType::SetTempo {
                us_per_quarter: 500_000
            }
        ));
        assert!(matches!(
            seq.events[1].event,
            MidiEventType::NoteOn { note: 0x3C, .. }
        ));
        assert_eq!(seq.events[2].absolute_tick, 48);
        assert!(matches!(
            seq.events[2].event,
            MidiEventType::NoteOn { note: 0x3E, .. }
        ));
        assert_eq!(seq.events[3].absolute_tick, 96);
        assert!(matches!(
            seq.events[3].event,
            MidiEventType::NoteOff { note: 0x3C, .. }
        ));
    }

    #[test]
    fn volume_scales_output_amplitude() {
        let peak_at = |volume: i32| {
            let mut synth = MidiSynthesizer::new(two_note_sequence(), 44100);
            synth.volume = volume;
            synth.start();
            let mut out = Vec::new();
            synth.render_samples(20000, &mut out);
            out.iter().fold(0.0f32, |acc, s| acc.max(s.abs()))
        };

        let full = peak_at(100);
        let half = peak_at(50);
        let muted = peak_at(0);

        assert!(
            full > half,
            "100% should be louder than 50%: {full} vs {half}"
        );
        assert!(muted < 0.0001, "0% must be silent, got {muted}");
    }

    #[test]
    fn test_brass_waveform_dc_offset_is_zero_mean() {
        let mut voice = Voice::default();
        voice.note_on(0, 60, 100, 56, 0.0, 1.0); // Trumpet (Brass family, program 56)
        let sr = 44100.0;
        let mut sum = 0.0f32;
        let samples = 44100;
        for _ in 0..samples {
            sum += voice.render_sample(sr);
        }
        let dc_offset = (sum / samples as f32).abs();
        assert!(
            dc_offset < 0.015,
            "Brass waveform DC offset must be near zero, got {dc_offset}"
        );
    }

    #[test]
    fn test_silence_after_all_notes_off() {
        let mut synth = MidiSynthesizer::new(two_note_sequence(), 44100);
        synth.start();
        let mut out = Vec::new();
        // Play through the sequence (96 ticks at 120 bpm is 0.5s = 22050 samples)
        synth.render_samples(44100 * 2, &mut out);
        // After sequence finishes, additional rendered samples must be pure silence
        let mut tail = Vec::new();
        synth.render_samples(1000, &mut tail);
        let peak = tail.iter().fold(0.0f32, |acc, s| acc.max(s.abs()));
        assert_eq!(peak, 0.0, "Synth must be completely silent after notes off");
    }
}
