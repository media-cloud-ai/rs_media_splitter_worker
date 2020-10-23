#[macro_use]
extern crate serde_derive;

mod duration;
mod message;
mod split_policy;

use duration::{Duration, DurationPosition};
use mcai_worker_sdk::{
  job::JobResult, start_worker, JsonSchema, McaiChannel, MessageError, MessageEvent, Version,
};

macro_rules! crate_version {
  () => {
    env!("CARGO_PKG_VERSION")
  };
}

#[derive(Debug, Default)]
struct MediaSplitterEvent {}

fn default_output_parameter_name() -> String {
  "segments".to_string()
}

fn default_segments() -> u64 {
  1
}

#[derive(Clone, Default, Debug, Deserialize, JsonSchema)]
pub struct MediaSplitterParameters {
  source_path: String,
  #[serde(default = "default_output_parameter_name")]
  output_parameter_name: String,

  /// Number of parts to split into
  #[serde(default = "default_segments")]
  number_of_segments: u64,
  /// Limit the minimal duration of a segment.  
  /// It overload the `segments` constraint.  
  min_segment_duration: Option<Duration>,
  /// Process only the part beginning after that entry point.
  entry_point: Option<Duration>,
  /// It will represent the duration of the content processed.
  duration: Option<Duration>,
  /// Overload the duration field.  
  /// Useful to limit duration to a maximum value.
  max_duration: Option<Duration>,
  /// Specify the position from which the selected duration is reckoned.  
  /// By default, it is set from the start of the file, but it can also be set from the end.
  #[serde(default = "DurationPosition::default")]
  duration_position: DurationPosition,
  /// It will add duration to overlap segments.  
  /// This means some data will be process twice.  
  overlap: Option<Duration>,
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
    message::process(channel, &parameters, job_result)
  }
}

fn main() {
  let message_event = MediaSplitterEvent::default();
  start_worker(message_event);
}
