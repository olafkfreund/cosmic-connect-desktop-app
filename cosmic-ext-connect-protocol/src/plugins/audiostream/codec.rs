//! Audio codec implementations
//!
//! Provides encoding and decoding for various audio codecs.

#[cfg(feature = "opus")]
use opus::{Channels as OpusChannels, Decoder as OpusDecoder, Encoder as OpusEncoder};
use tracing::debug;

use crate::{ProtocolError, Result};

use super::audio_backend::AudioSample;

/// Opus codec wrapper
///
/// # Safety
/// The Opus encoder/decoder contain raw pointers that aren't `Send` by default.
/// We implement `Send` because access is protected by `RwLock` in the plugin,
/// ensuring only one thread accesses the codec at a time.
#[cfg(feature = "opus")]
pub struct OpusCodec {
    encoder: OpusEncoder,
    decoder: OpusDecoder,
    sample_rate: u32,
    channels: u8,
    frame_size: usize,
}

// SAFETY: OpusCodec is protected by RwLock in AudioStreamPlugin,
// ensuring exclusive mutable access. The raw pointers in Opus
// encoder/decoder are only accessed through &mut self methods.
// We implement both Send and Sync because async functions that take
// &mut self across await points require the type to be Send + Sync.
#[cfg(feature = "opus")]
unsafe impl Send for OpusCodec {}
#[cfg(feature = "opus")]
unsafe impl Sync for OpusCodec {}

/// Stub Opus codec when feature is disabled
#[cfg(not(feature = "opus"))]
#[allow(dead_code)]
pub struct OpusCodec {
    sample_rate: u32,
    channels: u8,
    frame_size: usize,
}

#[cfg(feature = "opus")]
impl OpusCodec {
    /// Create new Opus codec
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz (8000, 16000, 24000, 48000)
    /// * `channels` - Number of channels (1=mono, 2=stereo)
    /// * `bitrate` - Target bitrate in bits per second
    pub fn new(sample_rate: u32, channels: u8, bitrate: u32) -> Result<Self> {
        // Validate sample rate
        if ![8000, 16000, 24000, 48000].contains(&sample_rate) {
            return Err(ProtocolError::InvalidPacket(format!(
                "Unsupported sample rate: {}. Must be 8000, 16000, 24000, or 48000",
                sample_rate
            )));
        }

        // Convert channels to Opus enum
        let opus_channels = match channels {
            1 => OpusChannels::Mono,
            2 => OpusChannels::Stereo,
            _ => {
                return Err(ProtocolError::InvalidPacket(format!(
                    "Unsupported channel count: {}. Must be 1 or 2",
                    channels
                )))
            }
        };

        // Create encoder and decoder
        let mut encoder = OpusEncoder::new(
            sample_rate,
            opus_channels,
            opus::Application::Voip, // Optimized for voice/communication
        )
        .map_err(|e| {
            ProtocolError::InvalidPacket(format!("Failed to create Opus encoder: {:?}", e))
        })?;

        encoder
            .set_bitrate(opus::Bitrate::Bits(bitrate as i32))
            .map_err(|e| {
                ProtocolError::InvalidPacket(format!("Failed to set Opus bitrate: {:?}", e))
            })?;

        let decoder = OpusDecoder::new(sample_rate, opus_channels).map_err(|e| {
            ProtocolError::InvalidPacket(format!("Failed to create Opus decoder: {:?}", e))
        })?;

        // Calculate frame size (samples per channel)
        // Opus typically uses 20ms frames
        let frame_size = (sample_rate as usize * 20) / 1000;

        debug!(
            "Created Opus codec: {}Hz, {} channels, {} bps, {} samples/frame",
            sample_rate, channels, bitrate, frame_size
        );

        Ok(Self {
            encoder,
            decoder,
            sample_rate,
            channels,
            frame_size,
        })
    }

    /// Encode audio samples to Opus
    ///
    /// # Arguments
    /// * `samples` - Interleaved f32 audio samples
    ///
    /// # Returns
    /// Encoded opus packet as bytes
    pub fn encode(&mut self, samples: &[AudioSample]) -> Result<Vec<u8>> {
        // Check if we have enough samples for a frame
        let expected_samples = self.frame_size * self.channels as usize;
        if samples.len() < expected_samples {
            return Err(ProtocolError::InvalidPacket(format!(
                "Not enough samples for encoding: got {}, expected {}",
                samples.len(),
                expected_samples
            )));
        }

        // Convert f32 samples to i16 for Opus
        let pcm_samples: Vec<i16> = samples[..expected_samples]
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();

        // Prepare output buffer (max opus packet is typically ~1275 bytes)
        let mut output = vec![0u8; 4000];

        // Encode
        let encoded_size = self
            .encoder
            .encode(&pcm_samples, &mut output)
            .map_err(|e| ProtocolError::InvalidPacket(format!("Opus encoding failed: {:?}", e)))?;

        output.truncate(encoded_size);

        debug!(
            "Encoded {} samples to {} bytes",
            samples.len(),
            encoded_size
        );

        Ok(output)
    }

    /// Decode Opus packet to audio samples
    ///
    /// # Arguments
    /// * `packet` - Encoded opus packet
    ///
    /// # Returns
    /// Decoded audio samples as interleaved f32
    pub fn decode(&mut self, packet: &[u8]) -> Result<Vec<AudioSample>> {
        // Prepare output buffer
        let output_samples = self.frame_size * self.channels as usize;
        let mut pcm_output = vec![0i16; output_samples];

        // Decode
        let decoded_samples = self
            .decoder
            .decode(packet, &mut pcm_output, false)
            .map_err(|e| ProtocolError::InvalidPacket(format!("Opus decoding failed: {:?}", e)))?;

        // Convert i16 to f32
        let samples: Vec<AudioSample> = pcm_output[..decoded_samples * self.channels as usize]
            .iter()
            .map(|&s| s as f32 / 32767.0)
            .collect();

        debug!(
            "Decoded {} bytes to {} samples",
            packet.len(),
            samples.len()
        );

        Ok(samples)
    }

    /// Decode with packet loss concealment
    ///
    /// Used when a packet is lost to generate placeholder audio
    pub fn decode_plc(&mut self) -> Result<Vec<AudioSample>> {
        // Prepare output buffer
        let output_samples = self.frame_size * self.channels as usize;
        let mut pcm_output = vec![0i16; output_samples];

        // Decode with FEC (forward error correction) - use empty slice as "lost" packet
        let decoded_samples = self
            .decoder
            .decode(&[], &mut pcm_output, true) // FEC flag enabled
            .map_err(|e| {
                ProtocolError::InvalidPacket(format!("Opus PLC decoding failed: {:?}", e))
            })?;

        // Convert i16 to f32
        let samples: Vec<AudioSample> = pcm_output[..decoded_samples * self.channels as usize]
            .iter()
            .map(|&s| s as f32 / 32767.0)
            .collect();

        debug!("Generated {} PLC samples", samples.len());

        Ok(samples)
    }

    /// Get frame size in samples per channel
    #[allow(dead_code)]
    pub fn frame_size(&self) -> usize {
        self.frame_size
    }

    /// Get sample rate
    #[allow(dead_code)]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get channel count
    #[allow(dead_code)]
    pub fn channels(&self) -> u8 {
        self.channels
    }
}

#[cfg(not(feature = "opus"))]
impl OpusCodec {
    /// Create new Opus codec (stub - always fails)
    pub fn new(_sample_rate: u32, _channels: u8, _bitrate: u32) -> Result<Self> {
        Err(ProtocolError::InvalidPacket(
            "Opus codec not available - compile with 'opus' feature and install libopus-dev"
                .to_string(),
        ))
    }

    /// Encode (stub)
    pub fn encode(&mut self, _samples: &[AudioSample]) -> Result<Vec<u8>> {
        Err(ProtocolError::InvalidPacket(
            "Opus codec not available".to_string(),
        ))
    }

    /// Decode (stub)
    pub fn decode(&mut self, _packet: &[u8]) -> Result<Vec<AudioSample>> {
        Err(ProtocolError::InvalidPacket(
            "Opus codec not available".to_string(),
        ))
    }

    /// Decode PLC (stub)
    pub fn decode_plc(&mut self) -> Result<Vec<AudioSample>> {
        Err(ProtocolError::InvalidPacket(
            "Opus codec not available".to_string(),
        ))
    }

    /// Get frame size in samples per channel
    #[allow(dead_code)]
    pub fn frame_size(&self) -> usize {
        960 // 20ms at 48kHz
    }

    /// Get sample rate
    #[allow(dead_code)]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get channel count
    #[allow(dead_code)]
    pub fn channels(&self) -> u8 {
        self.channels
    }
}

/// PCM codec (uncompressed)
#[allow(dead_code)]
pub struct PcmCodec {
    sample_rate: u32,
    channels: u8,
}

impl PcmCodec {
    /// Create new PCM codec
    pub fn new(sample_rate: u32, channels: u8) -> Self {
        debug!(
            "Created PCM codec: {}Hz, {} channels",
            sample_rate, channels
        );
        Self {
            sample_rate,
            channels,
        }
    }

    /// Encode audio samples to PCM (just convert f32 to i16 bytes)
    pub fn encode(&self, samples: &[AudioSample]) -> Result<Vec<u8>> {
        let mut output = Vec::with_capacity(samples.len() * 2);

        for &sample in samples {
            let s16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            output.extend_from_slice(&s16.to_le_bytes());
        }

        Ok(output)
    }

    /// Decode PCM bytes to audio samples
    pub fn decode(&self, data: &[u8]) -> Result<Vec<AudioSample>> {
        if data.len() % 2 != 0 {
            return Err(ProtocolError::InvalidPacket(
                "Invalid PCM data length (must be even)".to_string(),
            ));
        }

        let samples: Vec<AudioSample> = data
            .chunks_exact(2)
            .map(|chunk| {
                let s16 = i16::from_le_bytes([chunk[0], chunk[1]]);
                s16 as f32 / 32767.0
            })
            .collect();

        Ok(samples)
    }

    /// Get sample rate
    #[allow(dead_code)]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get channel count
    #[allow(dead_code)]
    pub fn channels(&self) -> u8 {
        self.channels
    }
}

/// AAC codec wrapper
#[cfg(feature = "aac")]
pub struct AacCodec {
    sample_rate: u32,
    channels: u8,
    bitrate: u32,
    frame_size: usize,
}

/// Stub AAC codec when feature is disabled
#[cfg(not(feature = "aac"))]
#[allow(dead_code)]
pub struct AacCodec {
    sample_rate: u32,
    channels: u8,
    bitrate: u32,
    frame_size: usize,
}

#[cfg(feature = "aac")]
impl AacCodec {
    /// Create new AAC codec
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz (8000, 16000, 24000, 48000)
    /// * `channels` - Number of channels (1=mono, 2=stereo)
    /// * `bitrate` - Target bitrate in bits per second
    pub fn new(sample_rate: u32, channels: u8, bitrate: u32) -> Result<Self> {
        // Validate sample rate
        if ![8000, 16000, 24000, 48000].contains(&sample_rate) {
            return Err(ProtocolError::InvalidPacket(format!(
                "Unsupported sample rate: {}. Must be 8000, 16000, 24000, or 48000",
                sample_rate
            )));
        }

        // Validate channels
        if channels < 1 || channels > 2 {
            return Err(ProtocolError::InvalidPacket(format!(
                "Unsupported channel count: {}. Must be 1 or 2",
                channels
            )));
        }

        // AAC typically uses 1024 samples per frame for most sample rates
        let frame_size = match sample_rate {
            8000 => 256,
            16000 => 512,
            24000 => 768,
            48000 => 1024,
            _ => 1024,
        };

        debug!(
            "Created AAC codec: {}Hz, {} channels, {} bps, {} samples/frame",
            sample_rate, channels, bitrate, frame_size
        );

        Ok(Self {
            sample_rate,
            channels,
            bitrate,
            frame_size,
        })
    }

    /// Encode audio samples to AAC
    ///
    /// # Arguments
    /// * `samples` - Interleaved f32 audio samples
    ///
    /// # Returns
    /// Encoded AAC packet as bytes
    ///
    /// # Implementation Notes
    ///
    /// This method is not yet fully implemented. AAC encoding requires integration
    /// with an external AAC encoder library.
    ///
    /// ## Recommended Implementation Approaches
    ///
    /// ### Option 1: fdk-aac (Fraunhofer FDK AAC)
    ///
    /// The industry-standard AAC encoder/decoder library.
    ///
    /// **Rust Bindings:**
    /// - `fdk-aac-sys`: Low-level FFI bindings to libfdk-aac
    /// - `fdk-aac`: Higher-level Rust wrapper (may need custom wrapper)
    ///
    /// **System Dependencies:**
    /// ```bash
    /// # Debian/Ubuntu
    /// sudo apt-get install libfdk-aac-dev
    ///
    /// # macOS
    /// brew install fdk-aac
    /// ```
    ///
    /// **Key Functions:**
    /// - `aacEncOpen()`: Create encoder instance
    /// - `aacEncoder_SetParam()`: Configure bitrate, sample rate, channels
    /// - `aacEncEncode()`: Encode audio frames
    /// - `aacEncClose()`: Clean up encoder
    ///
    /// ### Option 2: FFmpeg's libavcodec
    ///
    /// Using FFmpeg's AAC encoder through Rust bindings.
    ///
    /// **Rust Bindings:**
    /// - `ffmpeg-next`: Safe Rust bindings to FFmpeg
    /// - More features but heavier dependency
    ///
    /// ### Option 3: Native Rust Implementation
    ///
    /// Currently no production-ready pure Rust AAC encoder exists.
    /// This would be a significant undertaking.
    ///
    /// ## Implementation Pattern (fdk-aac example)
    ///
    /// ```ignore
    /// use fdk_aac_sys::*;
    ///
    /// // 1. Create encoder handle
    /// let mut encoder: HANDLE_AACENCODER = std::ptr::null_mut();
    /// aacEncOpen(&mut encoder, 0, self.channels as u32);
    ///
    /// // 2. Configure encoder
    /// aacEncoder_SetParam(encoder, AACENC_AOT, AOT_AAC_LC as i32); // AAC-LC profile
    /// aacEncoder_SetParam(encoder, AACENC_SAMPLERATE, self.sample_rate as i32);
    /// aacEncoder_SetParam(encoder, AACENC_CHANNELMODE, MODE_2 as i32); // Stereo
    /// aacEncoder_SetParam(encoder, AACENC_BITRATE, self.bitrate as i32);
    /// aacEncEncode(encoder, std::ptr::null(), std::ptr::null_mut(), &mut out_args);
    ///
    /// // 3. Convert f32 samples to i16
    /// let pcm_samples: Vec<i16> = samples.iter()
    ///     .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
    ///     .collect();
    ///
    /// // 4. Setup buffers and encode
    /// // ... (buffer configuration omitted for brevity)
    /// aacEncEncode(encoder, &mut in_args, &mut out_args);
    ///
    /// // 5. Collect output and cleanup
    /// // ... (output handling and aacEncClose)
    /// ```
    ///
    /// ## Testing Considerations
    ///
    /// - Verify output with standard AAC decoders (ffplay, VLC)
    /// - Test various bitrates (64kbps - 320kbps)
    /// - Validate frame alignment and sample rates
    /// - Handle encoder initialization failures gracefully
    pub fn encode(&mut self, samples: &[AudioSample]) -> Result<Vec<u8>> {
        // Check if we have enough samples for a frame
        let expected_samples = self.frame_size * self.channels as usize;
        if samples.len() < expected_samples {
            return Err(ProtocolError::InvalidPacket(format!(
                "Not enough samples for encoding: got {}, expected {}",
                samples.len(),
                expected_samples
            )));
        }

        // AAC encoding requires fdk-aac library integration (see implementation notes above)
        Err(ProtocolError::InvalidPacket(
            "AAC encoding not yet implemented - requires fdk-aac library integration".to_string(),
        ))
    }

    /// Decode AAC packet to audio samples
    ///
    /// # Arguments
    /// * `packet` - Encoded AAC packet
    ///
    /// # Returns
    /// Decoded audio samples as interleaved f32
    ///
    /// # Implementation Notes
    ///
    /// This method is not yet fully implemented. AAC decoding requires integration
    /// with an external AAC decoder library.
    ///
    /// ## Recommended Implementation Approaches
    ///
    /// ### Option 1: fdk-aac (Fraunhofer FDK AAC)
    ///
    /// Same library as encoding, provides both encoder and decoder.
    ///
    /// **Key Decoder Functions:**
    /// - `aacDecoder_Open()`: Create decoder instance
    /// - `aacDecoder_ConfigRaw()`: Configure decoder with audio info
    /// - `aacDecoder_Fill()`: Feed encoded data to decoder
    /// - `aacDecoder_DecodeFrame()`: Decode one frame
    /// - `aacDecoder_Close()`: Clean up decoder
    ///
    /// ### Option 2: FFmpeg's libavcodec
    ///
    /// More robust error handling and format support.
    ///
    /// ### Option 3: mp4parse + External Decoder
    ///
    /// Parse AAC bitstream with pure Rust, decode with C library.
    ///
    /// ## Implementation Pattern (fdk-aac example)
    ///
    /// ```ignore
    /// use fdk_aac_sys::*;
    ///
    /// // 1. Create decoder handle
    /// let decoder = aacDecoder_Open(TT_MP4_ADTS, 1); // ADTS transport format
    ///
    /// // 2. Feed input data
    /// let mut bytes_valid = packet.len();
    /// let mut input_ptr = packet.as_ptr();
    /// aacDecoder_Fill(decoder, &mut input_ptr, &packet.len(), &mut bytes_valid);
    ///
    /// // 3. Decode frame
    /// let mut output_buffer = vec![0i16; self.frame_size * self.channels as usize];
    /// aacDecoder_DecodeFrame(
    ///     decoder,
    ///     output_buffer.as_mut_ptr(),
    ///     output_buffer.len() as i32,
    ///     0
    /// );
    ///
    /// // 4. Convert i16 to f32
    /// let samples: Vec<f32> = output_buffer.iter()
    ///     .map(|&s| s as f32 / 32767.0)
    ///     .collect();
    ///
    /// // 5. Cleanup
    /// aacDecoder_Close(decoder);
    /// ```
    ///
    /// ## Error Handling
    ///
    /// - Validate packet size before decoding
    /// - Handle decoder errors (corrupted packets, format mismatches)
    /// - Implement packet loss concealment (silence or repeat last frame)
    /// - Reset decoder on stream discontinuities
    ///
    /// ## Testing Considerations
    ///
    /// - Test with packets generated by the encoder
    /// - Validate with standard AAC test vectors
    /// - Test error recovery (missing packets, corrupted data)
    /// - Verify sample rate and channel count match configuration
    pub fn decode(&mut self, _packet: &[u8]) -> Result<Vec<AudioSample>> {
        // AAC decoding requires fdk-aac library integration (see implementation notes above)
        Err(ProtocolError::InvalidPacket(
            "AAC decoding not yet implemented - requires fdk-aac library integration".to_string(),
        ))
    }

    /// Get frame size in samples per channel
    #[allow(dead_code)]
    pub fn frame_size(&self) -> usize {
        self.frame_size
    }

    /// Get sample rate
    #[allow(dead_code)]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get channel count
    #[allow(dead_code)]
    pub fn channels(&self) -> u8 {
        self.channels
    }

    /// Get bitrate
    #[allow(dead_code)]
    pub fn bitrate(&self) -> u32 {
        self.bitrate
    }
}

#[cfg(not(feature = "aac"))]
impl AacCodec {
    /// Create new AAC codec (stub - always fails)
    pub fn new(_sample_rate: u32, _channels: u8, _bitrate: u32) -> Result<Self> {
        Err(ProtocolError::InvalidPacket(
            "AAC codec not available - compile with 'aac' feature and install required libraries"
                .to_string(),
        ))
    }

    /// Encode (stub)
    pub fn encode(&mut self, _samples: &[AudioSample]) -> Result<Vec<u8>> {
        Err(ProtocolError::InvalidPacket(
            "AAC codec not available".to_string(),
        ))
    }

    /// Decode (stub)
    pub fn decode(&mut self, _packet: &[u8]) -> Result<Vec<AudioSample>> {
        Err(ProtocolError::InvalidPacket(
            "AAC codec not available".to_string(),
        ))
    }

    /// Get frame size in samples per channel
    #[allow(dead_code)]
    pub fn frame_size(&self) -> usize {
        1024 // Default AAC frame size at 48kHz
    }

    /// Get sample rate
    #[allow(dead_code)]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get channel count
    #[allow(dead_code)]
    pub fn channels(&self) -> u8 {
        self.channels
    }

    /// Get bitrate
    #[allow(dead_code)]
    pub fn bitrate(&self) -> u32 {
        self.bitrate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "opus")]
    fn test_opus_codec_creation() {
        let codec = OpusCodec::new(48000, 2, 128000);
        assert!(codec.is_ok());

        let codec = codec.unwrap();
        assert_eq!(codec.sample_rate(), 48000);
        assert_eq!(codec.channels(), 2);
    }

    #[test]
    #[cfg(feature = "opus")]
    fn test_opus_encode_decode() {
        let mut codec = OpusCodec::new(48000, 2, 128000).unwrap();

        // Generate test audio (1 frame = 20ms at 48kHz = 960 samples per channel)
        let frame_samples = codec.frame_size() * codec.channels() as usize;
        let samples: Vec<f32> = (0..frame_samples)
            .map(|i| ((i as f32 * 440.0 * 2.0 * std::f32::consts::PI) / 48000.0).sin() * 0.5)
            .collect();

        // Encode
        let encoded = codec.encode(&samples);
        assert!(encoded.is_ok());
        let encoded = encoded.unwrap();
        assert!(!encoded.is_empty());

        // Decode
        let decoded = codec.decode(&encoded);
        assert!(decoded.is_ok());
        let decoded = decoded.unwrap();
        assert_eq!(decoded.len(), samples.len());
    }

    #[test]
    #[cfg(feature = "opus")]
    fn test_opus_plc() {
        let mut codec = OpusCodec::new(48000, 2, 128000).unwrap();

        let plc_samples = codec.decode_plc();
        assert!(plc_samples.is_ok());

        let samples = plc_samples.unwrap();
        assert_eq!(
            samples.len(),
            codec.frame_size() * codec.channels() as usize
        );
    }

    #[test]
    fn test_pcm_codec() {
        let codec = PcmCodec::new(48000, 2);

        let samples: Vec<f32> = (0..960).map(|i| (i as f32 / 960.0) * 2.0 - 1.0).collect();

        let encoded = codec.encode(&samples).unwrap();
        assert_eq!(encoded.len(), samples.len() * 2);

        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), samples.len());

        // Check values are close (allow for quantization error)
        for (original, decoded) in samples.iter().zip(decoded.iter()) {
            assert!((original - decoded).abs() < 0.0001);
        }
    }

    #[test]
    fn test_aac_codec_creation_without_feature() {
        // Without aac feature, codec creation should fail
        let codec = AacCodec::new(48000, 2, 128000);
        assert!(codec.is_err());
    }

    #[test]
    #[cfg(feature = "aac")]
    fn test_aac_codec_creation() {
        let codec = AacCodec::new(48000, 2, 128000);
        assert!(codec.is_ok());

        let codec = codec.unwrap();
        assert_eq!(codec.sample_rate(), 48000);
        assert_eq!(codec.channels(), 2);
        assert_eq!(codec.bitrate(), 128000);
        assert_eq!(codec.frame_size(), 1024);
    }

    #[test]
    #[cfg(feature = "aac")]
    fn test_aac_invalid_sample_rate() {
        let codec = AacCodec::new(44100, 2, 128000);
        assert!(codec.is_err());
    }
}
