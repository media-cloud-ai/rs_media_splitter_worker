use amqp_worker::worker::{Parameter, ParameterType};
use amqp_worker::{
  job::{Job, JobResult},
  start_worker, MessageError, MessageEvent,
};
use lapin_futures::Channel;
use semver::Version;

#[macro_use]
extern crate log;

mod message;

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
    semver::Version::parse(crate_version!()).expect("unable to locate Package version")
  }

  fn get_parameters(&self) -> Vec<Parameter> {
    vec![
      Parameter {
        identifier: "source_path".to_string(),
        label: "Source Path".to_string(),
        kind: vec![ParameterType::String],
        required: true,
      },
      Parameter {
        identifier: "segment_duration".to_string(),
        label: "Segment duration in milliseconds".to_string(),
        kind: vec![ParameterType::Integer],
        required: true,
      },
      Parameter {
        identifier: "segment_overlap".to_string(),
        label: "Segment overlap duration in milliseconds".to_string(),
        kind: vec![ParameterType::Integer],
        required: false,
      },
    ]
  }

  fn process(
    &self,
    channel: Option<&Channel>,
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
