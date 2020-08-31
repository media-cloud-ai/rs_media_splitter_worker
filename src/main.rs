#[macro_use]
extern crate serde_derive;

use mcai_worker_sdk::{
  job::JobResult, start_worker, JsonSchema, McaiChannel, MessageError, MessageEvent, Version,
};

mod message;
mod split_policy;

macro_rules! crate_version {
  () => {
    env!("CARGO_PKG_VERSION")
  };
}

#[derive(Debug, Default)]
struct MediaSplitterEvent {}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub enum SegmentUnit {
  #[serde(rename = "milliseconds")]
  Milliseconds,
  #[serde(rename = "seconds")]
  Seconds,
  #[serde(rename = "segments")]
  Segments,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub enum OverlapUnit {
  #[serde(rename = "milliseconds")]
  Milliseconds,
  #[serde(rename = "seconds")]
  Seconds,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct MediaSplitterParameters {
  source_path: String,
  segments: u64,
  segments_unit: SegmentUnit,
  output_parameter_name: Option<String>,
  overlap: Option<i64>,
  overlap_unit: Option<OverlapUnit>,
}

impl MessageEvent<MediaSplitterParameters> for MediaSplitterEvent {
  fn get_name(&self) -> String {
    "Media splitter".to_string()
  }

  fn get_short_description(&self) -> String {
    "Split a media source into segments".to_string()
  }

  fn get_description(&self) -> String {
    r#"This worker split an audio/video media file into several segments.
These segment are defined by a duration in milliseconds, and they can overlap."#
      .to_string()
  }

  fn get_version(&self) -> Version {
    Version::parse(crate_version!()).expect("unable to locate Package version")
  }

  fn process(
    &self,
    channel: Option<McaiChannel>,
    parameters: MediaSplitterParameters,
    job_result: JobResult,
  ) -> Result<JobResult, MessageError> {
    message::process(channel, parameters, job_result)
  }
}

fn main() {
  let message_event = MediaSplitterEvent::default();
  start_worker(message_event);
}
