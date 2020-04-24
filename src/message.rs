use crate::split_policy::SplitPolicy;
use amqp_worker::{
  job::{Job, JobResult, JobStatus},
  MessageError, ParametersContainer,
};
use lapin_futures::Channel;
use stainless_ffmpeg::format_context::FormatContext;

const DEFAULT_OUTPUT_PARAMETER_NAME: &'static str = "segments";
const NUMBER_OF_SEGMENTS_PARAMETER: &'static str = "number_of_segments";
const OUTPUT_PARAMETER_NAME_PARAMETER: &'static str = "output_parameter_name";
const SEGMENT_DURATION_PARAMETER: &'static str = "segment_duration";
const SEGMENT_OVERLAP_PARAMETER: &'static str = "segment_overlap";
const SOURCE_PATH_PARAMETER: &'static str = "source_path";

pub fn process(
  _channel: Option<&Channel>,
  job: &Job,
  job_result: JobResult,
) -> Result<JobResult, MessageError> {
  let source_path = job
    .get_string_parameter(SOURCE_PATH_PARAMETER)
    .ok_or_else(|| {
      MessageError::ProcessingError(
        job_result
          .clone()
          .with_status(JobStatus::Error)
          .with_message(&format!(
            "Invalid job message: missing expected '{}' parameter.",
            SEGMENT_DURATION_PARAMETER
          )),
      )
    })?;

  let segment_duration_split_policy = job
    .get_integer_parameter(SEGMENT_DURATION_PARAMETER)
    .map(|segment_duration| SplitPolicy::SegmentDuration(segment_duration as u64));
  let number_of_segments_split_policy = job
    .get_integer_parameter(NUMBER_OF_SEGMENTS_PARAMETER)
    .map(|number_of_segments| SplitPolicy::NumberOfSegments(number_of_segments as u64));

  let output_parameter_name = job
    .get_string_parameter(OUTPUT_PARAMETER_NAME_PARAMETER)
    .unwrap_or(DEFAULT_OUTPUT_PARAMETER_NAME.to_string());

  let split_policy = segment_duration_split_policy
    .or(number_of_segments_split_policy)
    .ok_or_else(|| {
      MessageError::ProcessingError(
        job_result
          .clone()
          .with_status(JobStatus::Error)
          .with_message(&format!(
            "Invalid job message: missing '{}' or '{}' expected parameter.",
            SEGMENT_DURATION_PARAMETER, NUMBER_OF_SEGMENTS_PARAMETER
          )),
      )
    })?;

  let segment_overlap = job.get_integer_parameter(SEGMENT_OVERLAP_PARAMETER);

  let media_duration = get_media_duration_in_milliseconds(&source_path).map_err(|msg| {
    MessageError::ProcessingError(
      job_result
        .clone()
        .with_status(JobStatus::Error)
        .with_message(&msg),
    )
  })?;

  debug!("Input media duration: {} ms", media_duration);

  let segments = split_policy
    .split(media_duration, segment_overlap)
    .map_err(|msg| {
      MessageError::ProcessingError(
        job_result
          .clone()
          .with_status(JobStatus::Error)
          .with_message(&msg),
      )
    })?;

  let result = serde_json::to_string(&segments).map_err(|error| {
    MessageError::ProcessingError(
      job_result
        .clone()
        .with_status(JobStatus::Error)
        .with_message(&format!("{}", error)),
    )
  })?;

  Ok(
    job_result
      .with_status(JobStatus::Completed)
      .with_message(&result),
  )
}

fn get_media_duration_in_milliseconds(path: &str) -> Result<u64, String> {
  let mut format_context = FormatContext::new(path)?;
  format_context.open_input()?;

  let duration_millisec = format_context
    .get_duration()
    .map(|duration| (duration * 1000.) as u64)
    .unwrap_or_else(|| 0);

  format_context.close_input();
  Ok(duration_millisec)
}
