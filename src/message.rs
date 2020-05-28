use mcai_worker_sdk::job::{Job, JobResult, JobStatus};
use mcai_worker_sdk::{debug, warn};
use mcai_worker_sdk::{McaiChannel, MessageError, Parameter, ParametersContainer};
use stainless_ffmpeg::format_context::FormatContext;

use crate::split_policy::SplitPolicy;

pub const SOURCE_PATH_PARAMETER: &str = "source_path";

pub const OUTPUT_PARAMETER_NAME_PARAMETER: &str = "output_parameter_name";
pub const OUTPUT_PARAMETER_NAME_DEFAULT_VALUE: &str = "segments";

pub const SEGMENTS_PARAMETER: &str = "segments";
pub const SEGMENTS_UNIT_PARAMETER: &str = "segments_unit";
pub const SEGMENTS_UNIT_VALUES: [&str; 3] = ["segments", "seconds", "milliseconds"];

pub const OVERLAP_PARAMETER: &str = "overlap";
pub const OVERLAP_UNIT_PARAMETER: &str = "overlap_unit";
pub const OVERLAP_UNIT_VALUES: [&str; 2] = ["seconds", "milliseconds"];

pub fn process(
  _channel: Option<McaiChannel>,
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
            SOURCE_PATH_PARAMETER
          )),
      )
    })?;

  let segments_parameter = job
    .get_integer_parameter(SEGMENTS_PARAMETER)
    .ok_or_else(|| {
      MessageError::ProcessingError(
        job_result
          .clone()
          .with_status(JobStatus::Error)
          .with_message(&format!(
            "Invalid job message: missing expected '{}' parameter.",
            SEGMENTS_PARAMETER
          )),
      )
    })?;

  let split_policy = match job.get_string_parameter(SEGMENTS_UNIT_PARAMETER) {
    Some(param) if param == SEGMENTS_UNIT_VALUES[0] => {
      SplitPolicy::NumberOfSegments(segments_parameter as u64)
    }
    Some(param) if param == SEGMENTS_UNIT_VALUES[1] => {
      SplitPolicy::SegmentDurationSeconds(segments_parameter as u64)
    }
    Some(param) if param == SEGMENTS_UNIT_VALUES[2] => {
      SplitPolicy::SegmentDurationMilliSeconds(segments_parameter as u64)
    }
    other => {
      warn!(
        "Unspecified or invalid segments unit parameter: '{:?}'. Use default '{}' unit instead.",
        other, SEGMENTS_UNIT_VALUES[0]
      );
      SplitPolicy::NumberOfSegments(segments_parameter as u64)
    }
  };

  let segment_overlap = if let Some(overlap) = job.get_integer_parameter(OVERLAP_PARAMETER) {
    let milliseconds_overlap = match job.get_string_parameter(OVERLAP_UNIT_PARAMETER) {
      Some(param) if param == OVERLAP_UNIT_VALUES[0] => Ok(overlap * 1000),
      Some(param) if param == OVERLAP_UNIT_VALUES[1] => Ok(overlap),
      other => Err(MessageError::ProcessingError(
        job_result
          .clone()
          .with_status(JobStatus::Error)
          .with_message(&format!(
            "Invalid '{:?}' overlap unit. Possible values: {:?}",
            other, OVERLAP_UNIT_VALUES
          )),
      )),
    }?;
    Some(milliseconds_overlap)
  } else {
    None
  };

  let output_parameter_name = job
    .get_string_parameter(OUTPUT_PARAMETER_NAME_PARAMETER)
    .unwrap_or_else(|| OUTPUT_PARAMETER_NAME_DEFAULT_VALUE.to_string());

  let media_duration_in_milliseconds =
    get_media_duration_in_milliseconds(&source_path).map_err(|msg| {
      MessageError::ProcessingError(
        job_result
          .clone()
          .with_status(JobStatus::Error)
          .with_message(&msg),
      )
    })?;

  debug!(
    "Input media duration: {} ms",
    media_duration_in_milliseconds
  );

  let segments = split_policy
    .split(media_duration_in_milliseconds, segment_overlap)
    .map_err(|msg| {
      MessageError::ProcessingError(
        job_result
          .clone()
          .with_status(JobStatus::Error)
          .with_message(&msg),
      )
    })?;

  Ok(
    job_result
      .with_status(JobStatus::Completed)
      .with_parameters(&mut vec![Parameter::ArrayOfMediaSegmentsParam {
        id: output_parameter_name,
        value: Some(segments),
        default: Some(vec![]),
      }]),
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
