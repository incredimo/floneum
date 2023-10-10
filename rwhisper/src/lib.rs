//! # rwhisper
//! A rust wrapper for [whisper](https://openai.com/research/whisper)

#![warn(missing_docs)]

use cpal::FromSample;
use floneumin_streams::sender::ChannelTextStream;
use model::WhisperModelInner;
use rodio::{source::UniformSourceIterator, Source};
use std::fmt::Display;

use anyhow::Result;

use candle_transformers::models::whisper::{self as m};

use futures_util::{Stream, StreamExt};

mod model;
mod source;
pub use source::*;

#[derive(Debug, Clone)]
struct DecodingResult {
    text: String,
    avg_logprob: f64,
    no_speech_prob: f64,
    compression_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct Segment {
    start: f64,
    duration: f64,
    result: DecodingResult,
}

impl Segment {
    pub fn probability_of_no_speech(&self) -> f64 {
        self.result.no_speech_prob
    }

    pub fn text(&self) -> &str {
        &self.result.text
    }

    pub fn start(&self) -> f64 {
        self.start
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }
}

impl AsRef<str> for Segment {
    fn as_ref(&self) -> &str {
        self.text()
    }
}

#[async_trait::async_trait(?Send)]
pub trait TranscribeAudioSourceExt {
    fn text(self, model: WhisperModel) -> Result<ChannelTextStream<Segment>>;
}

#[async_trait::async_trait(?Send)]
impl<S: Source> TranscribeAudioSourceExt for S
where
    <S as Iterator>::Item: rodio::Sample,
    f32: FromSample<<S as Iterator>::Item>,
{
    fn text(self, model: WhisperModel) -> Result<ChannelTextStream<Segment>> {
        model.transcribe(self)
    }
}

pub trait TranscribeAudioStreamExt {
    fn text(self, model: WhisperModel) -> ChannelTextStream<Segment>;
}

impl<S> TranscribeAudioStreamExt for S
where
    S: Stream + std::marker::Unpin + Send + 'static,
    <S as Stream>::Item: Source + Send + 'static,
    <<S as Stream>::Item as Iterator>::Item: rodio::Sample,
    f32: FromSample<<<S as Stream>::Item as Iterator>::Item>,
{
    fn text(self, model: WhisperModel) -> ChannelTextStream<Segment> {
        let mut stream = self;
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(source) = stream.next().await {
                match model.transcribe(source) {
                    Ok(mut stream) => {
                        while let Some(segment) = stream.next().await {
                            if let Err(err) = sender.send(segment) {
                                tracing::error!("error sending segment: {}", err);
                                return;
                            }
                        }
                    }
                    Err(err) => tracing::error!("error transcribing audio: {}", err),
                }
            }
        });
        ChannelTextStream::from(receiver)
    }
}

#[derive(Clone, Copy, Debug)]
enum Task {
    Transcribe,
    #[allow(dead_code)]
    Translate,
}

#[derive(Debug)]
pub struct WhisperBuilder {
    /// Run on CPU rather than on GPU.
    cpu: bool,

    /// The model to be used, can be tiny, small, medium.
    model: WhisperModelSource,

    /// Language.
    language: Option<WhisperLanguage>,
}

impl Default for WhisperBuilder {
    fn default() -> Self {
        Self {
            cpu: false,
            model: WhisperModelSource::LargeV2,
            language: Some(WhisperLanguage::English),
        }
    }
}

impl WhisperBuilder {
    pub fn build(self) -> anyhow::Result<WhisperModel> {
        let (rx, tx) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap()
                .block_on(async move {
                    let mut model = WhisperModelInner::new(self).unwrap();
                    while let Ok(message) = tx.recv() {
                        match message {
                            WhisperMessage::Kill => return,
                            WhisperMessage::Transcribe(input, result) => {
                                model.transcribe(input, result);
                            }
                        }
                    }
                });
        });

        Ok(WhisperModel {
            thread: Some(thread),
            sender: rx,
        })
    }

    pub fn cpu(mut self, cpu: bool) -> Self {
        self.cpu = cpu;
        self
    }

    pub fn model(mut self, model: WhisperModelSource) -> Self {
        self.model = model;
        self
    }

    pub fn language(mut self, language: Option<WhisperLanguage>) -> Self {
        self.language = language;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WhisperLanguage {
    English,
    Chinese,
    German,
    Spanish,
    Russian,
    Korean,
    French,
    Japanese,
    Portuguese,
    Turkish,
    Polish,
    Catalan,
    Dutch,
    Arabic,
    Swedish,
    Italian,
    Indonesian,
    Hindi,
    Finnish,
    Vietnamese,
    Hebrew,
    Ukrainian,
    Greek,
    Malay,
    Czech,
    Romanian,
    Danish,
    Hungarian,
    Tamil,
    Norwegian,
    Thai,
    Urdu,
    Croatian,
    Bulgarian,
    Lithuanian,
    Latin,
    Maori,
    Malayalam,
    Welsh,
    Slovak,
    Telugu,
    Persian,
    Latvian,
    Bengali,
    Serbian,
    Azerbaijani,
    Slovenian,
    Kannada,
    Estonian,
    Macedonian,
    Breton,
    Basque,
    Icelandic,
    Armenian,
    Nepali,
    Mongolian,
    Bosnian,
    Kazakh,
    Albanian,
    Swahili,
    Galician,
    Marathi,
    Punjabi,
    Sinhala,
    Khmer,
    Shona,
    Yoruba,
    Somali,
    Afrikaans,
    Occitan,
    Georgian,
    Belarusian,
    Tajik,
    Sindhi,
    Gujarati,
    Amharic,
    Yiddish,
    Lao,
    Uzbek,
    Faroese,
    HaitianCreole,
    Pashto,
    Turkmen,
    Nynorsk,
    Maltese,
    Sanskrit,
    Luxembourgish,
    Myanmar,
    Tibetan,
    Tagalog,
    Malagasy,
    Assamese,
    Tatar,
    Hawaiian,
    Lingala,
    Hausa,
    Bashkir,
    Javanese,
    Sundanese,
}

impl Display for WhisperLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhisperLanguage::English => write!(f, "en"),
            WhisperLanguage::Chinese => write!(f, "zh"),
            WhisperLanguage::German => write!(f, "de"),
            WhisperLanguage::Spanish => write!(f, "es"),
            WhisperLanguage::Russian => write!(f, "ru"),
            WhisperLanguage::Korean => write!(f, "ko"),
            WhisperLanguage::French => write!(f, "fr"),
            WhisperLanguage::Japanese => write!(f, "ja"),
            WhisperLanguage::Portuguese => write!(f, "pt"),
            WhisperLanguage::Turkish => write!(f, "tr"),
            WhisperLanguage::Polish => write!(f, "pl"),
            WhisperLanguage::Catalan => write!(f, "ca"),
            WhisperLanguage::Dutch => write!(f, "nl"),
            WhisperLanguage::Arabic => write!(f, "ar"),
            WhisperLanguage::Swedish => write!(f, "sv"),
            WhisperLanguage::Italian => write!(f, "it"),
            WhisperLanguage::Indonesian => write!(f, "id"),
            WhisperLanguage::Hindi => write!(f, "hi"),
            WhisperLanguage::Finnish => write!(f, "fi"),
            WhisperLanguage::Vietnamese => write!(f, "vi"),
            WhisperLanguage::Hebrew => write!(f, "he"),
            WhisperLanguage::Ukrainian => write!(f, "uk"),
            WhisperLanguage::Greek => write!(f, "el"),
            WhisperLanguage::Malay => write!(f, "ms"),
            WhisperLanguage::Czech => write!(f, "cs"),
            WhisperLanguage::Romanian => write!(f, "ro"),
            WhisperLanguage::Danish => write!(f, "da"),
            WhisperLanguage::Hungarian => write!(f, "hu"),
            WhisperLanguage::Tamil => write!(f, "ta"),
            WhisperLanguage::Norwegian => write!(f, "no"),
            WhisperLanguage::Thai => write!(f, "th"),
            WhisperLanguage::Urdu => write!(f, "ur"),
            WhisperLanguage::Croatian => write!(f, "hr"),
            WhisperLanguage::Bulgarian => write!(f, "bg"),
            WhisperLanguage::Lithuanian => write!(f, "lt"),
            WhisperLanguage::Latin => write!(f, "la"),
            WhisperLanguage::Maori => write!(f, "mi"),
            WhisperLanguage::Malayalam => write!(f, "ml"),
            WhisperLanguage::Welsh => write!(f, "cy"),
            WhisperLanguage::Slovak => write!(f, "sk"),
            WhisperLanguage::Telugu => write!(f, "te"),
            WhisperLanguage::Persian => write!(f, "fa"),
            WhisperLanguage::Latvian => write!(f, "lv"),
            WhisperLanguage::Bengali => write!(f, "bn"),
            WhisperLanguage::Serbian => write!(f, "sr"),
            WhisperLanguage::Azerbaijani => write!(f, "az"),
            WhisperLanguage::Slovenian => write!(f, "sl"),
            WhisperLanguage::Kannada => write!(f, "kn"),
            WhisperLanguage::Estonian => write!(f, "et"),
            WhisperLanguage::Macedonian => write!(f, "mk"),
            WhisperLanguage::Breton => write!(f, "br"),
            WhisperLanguage::Basque => write!(f, "eu"),
            WhisperLanguage::Icelandic => write!(f, "is"),
            WhisperLanguage::Armenian => write!(f, "hy"),
            WhisperLanguage::Nepali => write!(f, "ne"),
            WhisperLanguage::Mongolian => write!(f, "mn"),
            WhisperLanguage::Bosnian => write!(f, "bs"),
            WhisperLanguage::Kazakh => write!(f, "kk"),
            WhisperLanguage::Albanian => write!(f, "sq"),
            WhisperLanguage::Swahili => write!(f, "sw"),
            WhisperLanguage::Galician => write!(f, "gl"),
            WhisperLanguage::Marathi => write!(f, "mr"),
            WhisperLanguage::Punjabi => write!(f, "pa"),
            WhisperLanguage::Sinhala => write!(f, "si"),
            WhisperLanguage::Khmer => write!(f, "km"),
            WhisperLanguage::Shona => write!(f, "sn"),
            WhisperLanguage::Yoruba => write!(f, "yo"),
            WhisperLanguage::Somali => write!(f, "so"),
            WhisperLanguage::Afrikaans => write!(f, "af"),
            WhisperLanguage::Occitan => write!(f, "oc"),
            WhisperLanguage::Georgian => write!(f, "ka"),
            WhisperLanguage::Belarusian => write!(f, "be"),
            WhisperLanguage::Tajik => write!(f, "tg"),
            WhisperLanguage::Sindhi => write!(f, "sd"),
            WhisperLanguage::Gujarati => write!(f, "gu"),
            WhisperLanguage::Amharic => write!(f, "am"),
            WhisperLanguage::Yiddish => write!(f, "yi"),
            WhisperLanguage::Lao => write!(f, "lo"),
            WhisperLanguage::Uzbek => write!(f, "uz"),
            WhisperLanguage::Faroese => write!(f, "fo"),
            WhisperLanguage::HaitianCreole => write!(f, "ht"),
            WhisperLanguage::Pashto => write!(f, "ps"),
            WhisperLanguage::Turkmen => write!(f, "tk"),
            WhisperLanguage::Nynorsk => write!(f, "nn"),
            WhisperLanguage::Maltese => write!(f, "mt"),
            WhisperLanguage::Sanskrit => write!(f, "sa"),
            WhisperLanguage::Luxembourgish => write!(f, "lb"),
            WhisperLanguage::Myanmar => write!(f, "my"),
            WhisperLanguage::Tibetan => write!(f, "bo"),
            WhisperLanguage::Tagalog => write!(f, "tl"),
            WhisperLanguage::Malagasy => write!(f, "mg"),
            WhisperLanguage::Assamese => write!(f, "as"),
            WhisperLanguage::Tatar => write!(f, "tt"),
            WhisperLanguage::Hawaiian => write!(f, "haw"),
            WhisperLanguage::Lingala => write!(f, "ln"),
            WhisperLanguage::Hausa => write!(f, "ha"),
            WhisperLanguage::Bashkir => write!(f, "ba"),
            WhisperLanguage::Javanese => write!(f, "jw"),
            WhisperLanguage::Sundanese => write!(f, "su"),
        }
    }
}

pub struct WhisperModel {
    thread: Option<std::thread::JoinHandle<()>>,
    sender: std::sync::mpsc::Sender<WhisperMessage>,
}

impl Default for WhisperModel {
    fn default() -> Self {
        Self::builder().build().unwrap()
    }
}

impl WhisperModel {
    pub fn builder() -> WhisperBuilder {
        WhisperBuilder::default()
    }

    pub fn transcribe<S: Source>(&self, input: S) -> Result<ChannelTextStream<Segment>>
    where
        <S as Iterator>::Item: rodio::Sample,
        f32: FromSample<<S as Iterator>::Item>,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let pcm_data: Vec<_> = normalize_audio(input)?;
        self.sender.send(WhisperMessage::Transcribe(pcm_data, tx))?;
        Ok(rx.into())
    }

    pub fn transcribe_into<S: Source>(
        &mut self,
        input: S,
        sender: tokio::sync::mpsc::UnboundedSender<Segment>,
    ) -> Result<()>
    where
        <S as Iterator>::Item: rodio::Sample,
        f32: FromSample<<S as Iterator>::Item>,
    {
        let pcm_data: Vec<_> = normalize_audio(input)?;
        self.sender
            .send(WhisperMessage::Transcribe(pcm_data, sender))?;
        Ok(())
    }
}

impl Drop for WhisperModel {
    fn drop(&mut self) {
        self.sender.send(WhisperMessage::Kill).unwrap();
        self.thread.take().unwrap().join().unwrap();
    }
}

enum WhisperMessage {
    Kill,
    Transcribe(Vec<f32>, tokio::sync::mpsc::UnboundedSender<Segment>),
}

pub fn normalize_audio<S: Source>(input: S) -> Result<Vec<f32>>
where
    <S as Iterator>::Item: rodio::Sample,
    f32: FromSample<<S as Iterator>::Item>,
{
    let resample = UniformSourceIterator::new(input, 1, m::SAMPLE_RATE as u32);
    let pass_filter = resample.low_pass(3000).high_pass(200).convert_samples();

    let samples = pass_filter.collect::<Vec<f32>>();

    Ok(samples)
}
