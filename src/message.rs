
use mcai_worker_sdk::{
  debug,
  job::{JobResult, JobStatus},
  McaiChannel, MessageError,
};
use stainless_ffmpeg::format_context::FormatContext;

use crate::split_policy::SplitPolicy;
use crate::{MediaSplitterParameters, OverlapUnit, SegmentUnit};

const OUTPUT_PARAMETER_NAME_DEFAULT_VALUE: &str = "segments";

pub fn process(
  _channel: Option<McaiChannel>,
  parameters: MediaSplitterParameters,
  job_result: JobResult,
) -> Result<JobResult, MessageError> {
  let split_policy = match parameters.segments_unit {
    SegmentUnit::Segments => SplitPolicy::NumberOfSegments(parameters.segments),
    SegmentUnit::Seconds => SplitPolicy::SegmentDurationSeconds(parameters.segments),
    SegmentUnit::Milliseconds => SplitPolicy::SegmentDurationMilliSeconds(parameters.segments),
  };

  let segment_overlap = if let Some(overlap) = parameters.overlap {
    let milliseconds_overlap = match parameters.overlap_unit {
      Some(OverlapUnit::Seconds) => Ok(overlap * 1000),
      Some(OverlapUnit::Milliseconds) => Ok(overlap),
      None => Err(MessageError::ProcessingError(
        job_result
          .clone()
          .with_status(JobStatus::Error)
          .with_message("Expected overlap unit."),
      )),
    }?;
    Some(milliseconds_overlap)
  } else {
    None
  };


  let output_parameter_name = parameters
    .output_parameter_name
    .unwrap_or_else(|| OUTPUT_PARAMETER_NAME_DEFAULT_VALUE.to_string());

  let media_duration_in_milliseconds = get_media_duration_in_milliseconds(&parameters.source_path)
    .map_err(|msg| {
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
      .with_json(&output_parameter_name, &segments)
      .map_err(MessageError::RuntimeError)?,
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
