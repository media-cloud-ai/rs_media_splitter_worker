use mcai_worker_sdk::job::{Job, JobResult};
use mcai_worker_sdk::start_worker;
use mcai_worker_sdk::worker::{Parameter, ParameterType};
use mcai_worker_sdk::{McaiChannel, MessageError, MessageEvent, Version};

mod message;
mod split_policy;

macro_rules! crate_version {
  () => {
    env!("CARGO_PKG_VERSION")
  };
}

#[derive(Debug)]
struct MediaSplitterEvent {}

impl MessageEvent for MediaSplitterEvent {
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

  fn get_parameters(&self) -> Vec<Parameter> {
    vec![
      Parameter {
        identifier: message::SOURCE_PATH_PARAMETER.to_string(),
        label: "Source path".to_string(),
        kind: vec![ParameterType::String],
        required: true,
      },
      Parameter {
        identifier: message::SEGMENTS_PARAMETER.to_string(),
        label: format!(
          "Number of segments (default) or segments duration, depending on '{}' parameter",
          message::SEGMENTS_UNIT_PARAMETER
        ),
        kind: vec![ParameterType::Integer],
        required: true,
      },
      Parameter {
        identifier: message::SEGMENTS_UNIT_PARAMETER.to_string(),
        label: format!(
          "Unit of the segments definition. Possible values: {:?}",
          message::SEGMENTS_UNIT_VALUES
        ),
        kind: vec![ParameterType::String],
        required: true,
      },
      Parameter {
        identifier: message::OVERLAP_PARAMETER.to_string(),
        label: "Segment overlap duration".to_string(),
        kind: vec![ParameterType::Integer],
        required: false,
      },
      Parameter {
        identifier: message::OVERLAP_UNIT_PARAMETER.to_string(),
        label: format!(
          "Unit of the segments overlap. Possible values: {:?}",
          message::OVERLAP_UNIT_VALUES
        ),
        kind: vec![ParameterType::String],
        required: false,
      },
      Parameter {
        identifier: message::OUTPUT_PARAMETER_NAME_PARAMETER.to_string(),
        label: format!(
          "Name of the output array of segments parameter. Default: '{}'",
          message::OUTPUT_PARAMETER_NAME_DEFAULT_VALUE
        ),
        kind: vec![ParameterType::String],
        required: false,
      },
    ]
  }

  fn process(
    &self,
    channel: Option<McaiChannel>,
    job: &Job,
    job_result: JobResult,
  ) -> Result<JobResult, MessageError> {
    message::process(channel, job, job_result)
  }
}

static MEDIA_SPLITTER_EVENT: MediaSplitterEvent = MediaSplitterEvent {};

fn main() {
  start_worker(&MEDIA_SPLITTER_EVENT);
}
