use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, SampleFormat, Stream, StreamError, SupportedStreamConfig};

use crate::errors::BeeperError;

pub struct Beeper {
    device: cpal::Device,
    supported_config: SupportedStreamConfig,
    stream: Stream,
    vol: f32
}
impl Beeper {
    pub fn new(vol: f32) -> Result<Self, BeeperError>  {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(BeeperError::NoDefaultOutputDevice)?;
        let supported_config = device.default_output_config().unwrap();
        let config = supported_config.config();
        let sample_format = supported_config.sample_format();

        let streamres = match sample_format {
            SampleFormat::F32 => run::<f32>(&device, &config, vol),
            SampleFormat::I16 => run::<i16>(&device, &config, vol),
            SampleFormat::U16 => run::<u16>(&device, &config, vol),
        }?;
        Ok(Self {
            device,
            supported_config,
            stream: streamres,
            vol,
        })
    }
    pub fn set_vol(&mut self, vol: f32) -> Result<(), BeeperError> {
        if self.vol != vol {
            let sample_format = self.supported_config.sample_format();
            let config = self.supported_config.config();
            self.stream = match sample_format {
                SampleFormat::F32 => run::<f32>(&self.device, &config, vol),
                SampleFormat::I16 => run::<i16>(&self.device, &config, vol),
                SampleFormat::U16 => run::<u16>(&self.device, &config, vol),
            }?;
            self.vol = vol;
        }
        Ok(())
    }
    pub fn play(&self) {
        self.stream.play().unwrap();
    }
    pub fn pause(&self) {
        self.stream.pause().unwrap();
    }
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, vol: f32) -> Result<Stream, BeeperError>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        ((sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin() / 6.0) * vol
    };

    #[cfg(not(target_arch = "wasm32"))]
    let err_fn = |err: StreamError| eprintln!("an error occurred on stream: {}", err);

    #[cfg(target_arch = "wasm32")]
    let err_fn = |err: StreamError| { gloo_console::log!("an error occurred on stream: {}", err.to_string()) };
    
    Ok(
        device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            err_fn,
        )?
    )
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}