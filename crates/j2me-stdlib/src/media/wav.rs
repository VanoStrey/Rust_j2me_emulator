/// RIFF / WAVE PCM Audio Decoder and Resampler
///
/// Supports 8-bit unsigned PCM and 16-bit signed PCM (mono and stereo),
/// with linear interpolation resampling to any target sample rate.

#[derive(Debug, Clone)]
pub struct WavDecoder {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub samples: Vec<f32>, // Normalized mono samples (-1.0 .. 1.0)
}

/// RIFF `fmt ` format tag for linear PCM. ADPCM (2), IEEE float (3) and
/// others are deliberately rejected instead of decoded as noise.
const WAVE_FORMAT_PCM: u16 = 1;

impl WavDecoder {
    pub fn parse(data: &[u8], target_sample_rate: u32) -> Result<Self, String> {
        if data.len() < 12 {
            return Err("WAV data too short for RIFF header".to_string());
        }

        if &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
            return Err("Invalid RIFF/WAVE magic bytes".to_string());
        }

        let mut offset = 12;
        let mut audio_format: u16 = 1;
        let mut channels: u16 = 1;
        let mut sample_rate: u32 = 44100;
        let mut bits_per_sample: u16 = 16;
        let mut raw_data: Option<&[u8]> = None;

        while offset + 8 <= data.len() {
            let chunk_id = &data[offset..offset + 4];
            let chunk_size = u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]) as usize;
            offset += 8;

            if offset + chunk_size > data.len() {
                // If chunk size extends beyond buffer, truncate to available bytes
                if chunk_id == b"data" {
                    raw_data = Some(&data[offset..]);
                }
                break;
            }

            if chunk_id == b"fmt " && chunk_size >= 16 {
                audio_format = u16::from_le_bytes([data[offset], data[offset + 1]]);
                channels = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
                sample_rate = u32::from_le_bytes([
                    data[offset + 4],
                    data[offset + 5],
                    data[offset + 6],
                    data[offset + 7],
                ]);
                bits_per_sample = u16::from_le_bytes([data[offset + 14], data[offset + 15]]);
            } else if chunk_id == b"data" {
                raw_data = Some(&data[offset..offset + chunk_size]);
            }

            // Subchunks are word-aligned
            offset += (chunk_size + 1) & !1;
        }

        let pcm_bytes = raw_data.ok_or_else(|| "WAV missing 'data' chunk".to_string())?;

        // Decode raw bytes into source mono f32 samples
        let src_samples = Self::decode_pcm(pcm_bytes, audio_format, channels, bits_per_sample)?;

        // Resample to target sample rate if needed
        let resampled = if sample_rate != target_sample_rate && !src_samples.is_empty() {
            Self::resample(&src_samples, sample_rate, target_sample_rate)
        } else {
            src_samples
        };

        Ok(Self {
            sample_rate: target_sample_rate,
            channels: 1,
            bits_per_sample: 16,
            samples: resampled,
        })
    }

    fn decode_pcm(data: &[u8], format: u16, channels: u16, bits: u16) -> Result<Vec<f32>, String> {
        if format != WAVE_FORMAT_PCM {
            return Err(format!(
                "unsupported WAV encoding (format tag {}), only PCM is decoded",
                format
            ));
        }
        let ch = channels.max(1) as usize;
        let mut mono_samples = Vec::new();

        match bits {
            8 => {
                let num_frames = data.len() / ch;
                mono_samples.reserve(num_frames);
                for frame in 0..num_frames {
                    let mut sum = 0.0f32;
                    for c in 0..ch {
                        let b = data[frame * ch + c];
                        // 8-bit unsigned: 128 is 0.0
                        let s = (b as f32 - 128.0) / 128.0;
                        sum += s;
                    }
                    mono_samples.push(sum / ch as f32);
                }
            }
            16 => {
                let frame_size = ch * 2;
                let num_frames = data.len() / frame_size;
                mono_samples.reserve(num_frames);
                for frame in 0..num_frames {
                    let mut sum = 0.0f32;
                    for c in 0..ch {
                        let base = frame * frame_size + c * 2;
                        let s = i16::from_le_bytes([data[base], data[base + 1]]);
                        sum += s as f32 / 32768.0;
                    }
                    mono_samples.push(sum / ch as f32);
                }
            }
            other => {
                return Err(format!(
                    "unsupported PCM bit depth {}: only 8 and 16 bit are decoded",
                    other
                ));
            }
        }

        Ok(mono_samples)
    }

    fn resample(src: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
        if src.is_empty() || src_rate == 0 || dst_rate == 0 {
            return Vec::new();
        }

        let ratio = src_rate as f64 / dst_rate as f64;
        let dst_len = ((src.len() as f64) / ratio).ceil() as usize;
        let mut out = Vec::with_capacity(dst_len);

        for i in 0..dst_len {
            let src_idx_f = i as f64 * ratio;
            let idx0 = src_idx_f.floor() as usize;
            let frac = (src_idx_f - idx0 as f64) as f32;

            if idx0 >= src.len() {
                break;
            }

            let s0 = src[idx0];
            let s1 = if idx0 + 1 < src.len() {
                src[idx0 + 1]
            } else {
                s0
            };
            out.push(s0 * (1.0 - frac) + s1 * frac);
        }

        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal RIFF/WAVE file around the given PCM payload.
    fn wav_file(
        format_tag: u16,
        channels: u16,
        sample_rate: u32,
        bits: u16,
        pcm: &[u8],
    ) -> Vec<u8> {
        let block_align = channels * (bits / 8);
        let byte_rate = sample_rate * block_align as u32;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        let riff_size = 36 + pcm.len() as u32;
        out.extend_from_slice(&riff_size.to_le_bytes());
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&format_tag.to_le_bytes());
        out.extend_from_slice(&channels.to_le_bytes());
        out.extend_from_slice(&sample_rate.to_le_bytes());
        out.extend_from_slice(&byte_rate.to_le_bytes());
        out.extend_from_slice(&block_align.to_le_bytes());
        out.extend_from_slice(&bits.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
        out.extend_from_slice(pcm);
        out
    }

    #[test]
    fn decodes_mono_16_bit_pcm() {
        let pcm = [0i16, i16::MAX, i16::MIN, 16384]
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect::<Vec<u8>>();
        let wav = wav_file(1, 1, 22050, 16, &pcm);
        let decoder = WavDecoder::parse(&wav, 22050).expect("valid 16-bit mono wav");

        assert_eq!(decoder.samples.len(), 4);
        assert_eq!(decoder.samples[0], 0.0);
        assert!((decoder.samples[1] - 1.0).abs() < 0.001);
        assert!((decoder.samples[2] + 1.0).abs() < 0.001);
        assert!((decoder.samples[3] - 0.5).abs() < 0.001);
    }

    #[test]
    fn stereo_is_downmixed_to_mono_average() {
        // Frame 1: L = +1.0, R = -1.0 -> 0.0
        // Frame 2: L = 0.5, R = 0.5  -> 0.5
        let pcm = [i16::MAX, i16::MIN, 16384, 16384]
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect::<Vec<u8>>();
        let wav = wav_file(1, 2, 44100, 16, &pcm);
        let decoder = WavDecoder::parse(&wav, 44100).expect("valid stereo wav");

        assert_eq!(decoder.samples.len(), 2);
        assert!(decoder.samples[0].abs() < 0.001);
        assert!((decoder.samples[1] - 0.5).abs() < 0.001);
    }

    #[test]
    fn decodes_8_bit_unsigned_pcm() {
        // 128 is silence, 255 is near +1.0, 0 is -1.0
        let pcm = [128u8, 255, 0];
        let wav = wav_file(1, 1, 8000, 8, &pcm);
        let decoder = WavDecoder::parse(&wav, 8000).expect("valid 8-bit wav");

        assert_eq!(decoder.samples.len(), 3);
        assert!(decoder.samples[0].abs() < 0.001);
        assert!((decoder.samples[1] - 0.9921875).abs() < 0.001);
        assert!((decoder.samples[2] + 1.0).abs() < 0.001);
    }

    #[test]
    fn rejects_non_pcm_encoding_instead_of_emitting_noise() {
        // ADPCM (format tag 2) must fail loudly, not decode as 8-bit garbage.
        let wav = wav_file(2, 1, 8000, 4, &[0u8; 16]);
        let err = WavDecoder::parse(&wav, 8000).expect_err("ADPCM must be rejected");
        assert!(
            err.contains("unsupported WAV encoding"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn rejects_unsupported_bit_depth() {
        let wav = wav_file(1, 1, 8000, 24, &[0u8; 12]);
        let err = WavDecoder::parse(&wav, 8000).expect_err("24-bit must be rejected");
        assert!(
            err.contains("unsupported PCM bit depth"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn rejects_bad_magic_and_missing_data_chunk() {
        assert!(WavDecoder::parse(b"NOTRIFF....WAVE", 8000).is_err());

        let mut wav = wav_file(1, 1, 8000, 16, &[0u8; 4]);
        // Rename the data chunk so no payload can be found.
        let pos = wav.windows(4).position(|w| w == b"data").unwrap();
        wav[pos..pos + 4].copy_from_slice(b"junk");
        assert!(WavDecoder::parse(&wav, 8000).is_err());
    }

    #[test]
    fn resamples_when_target_rate_differs() {
        let pcm = vec![0u8; 2 * 100]; // 100 silent 16-bit frames at 8000 Hz
        let wav = wav_file(1, 1, 8000, 16, &pcm);

        let same = WavDecoder::parse(&wav, 8000).unwrap();
        assert_eq!(same.samples.len(), 100);

        // Doubling the rate must roughly double the sample count, and the
        // decoder reports the requested rate, not the file's.
        let upsampled = WavDecoder::parse(&wav, 16000).unwrap();
        assert_eq!(upsampled.sample_rate, 16000);
        assert!(
            (upsampled.samples.len() as i64 - 200).abs() <= 2,
            "expected ~200 samples, got {}",
            upsampled.samples.len()
        );
        assert!(upsampled.samples.iter().all(|s| s.abs() < 0.001));
    }
}
