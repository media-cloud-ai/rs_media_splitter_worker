use mcai_worker_sdk::{
  debug,
  job::{JobResult, JobStatus},
  parameter::media_segment::MediaSegment,
  McaiChannel, MessageError,
};
use stainless_ffmpeg::format_context::FormatContext;
use std::cmp::min;

use crate::{duration::DurationPosition, split_policy::SplitPolicy, MediaSplitterParameters};

pub fn process(
  _channel: Option<McaiChannel>,
  parameters: &MediaSplitterParameters,
  job_result: JobResult,
) -> Result<JobResult, MessageError> {
  let media_duration =
    get_media_duration_in_milliseconds(&parameters.source_path).map_err(|msg| {
      MessageError::ProcessingError(
        job_result
          .clone()
          .with_status(JobStatus::Error)
          .with_message(&msg),
      )
    })?;

  debug!("Input media duration: {} ms", media_duration);

  let segments = generate_segments(parameters, media_duration)?;

  Ok(
    job_result
      .with_status(JobStatus::Completed)
      .with_json(&parameters.output_parameter_name, &segments)
      .map_err(MessageError::RuntimeError)?,
  )
}

fn generate_segments(
  parameters: &MediaSplitterParameters,
  media_duration: u64,
) -> Result<Vec<MediaSegment>, MessageError> {
  let total_duration = if let Some(duration) = &parameters.duration {
    min(duration.clone().to_millis(media_duration), media_duration)
  } else {
    media_duration
  };

  let total_duration = if let Some(max_duration) = &parameters.max_duration {
    min(
      max_duration.clone().to_millis(media_duration),
      total_duration,
    )
  } else {
    total_duration
  };

  let split_policy = SplitPolicy::new(parameters, total_duration);

  let segment_overlap = parameters
    .overlap
    .clone()
    .map(|overlap| overlap.to_millis(media_duration));

  let start_offset = match parameters.duration_position {
    DurationPosition::Start => 0,
    DurationPosition::End => media_duration - total_duration,
  };

  Ok(split_policy.split(total_duration, start_offset, segment_overlap))
}

fn get_media_duration_in_milliseconds(path: &str) -> Result<u64, String> {
  let mut format_context = FormatContext::new(path)?;
  format_context.open_input()?;

  let duration_millisec = format_context
    .get_duration()
    .map(|duration| duration as u64 * 1000)
    .unwrap_or_else(|| 0);

  format_context.close_input();
  Ok(duration_millisec)
}

#[test]
fn default() {
  let parameters = MediaSplitterParameters {
    source_path: "fake_source.mxf".to_string(),
    output_parameter_name: crate::default_output_parameter_name(),
    number_of_segments: 1,
    ..Default::default()
  };
  println!("{:?}", parameters);

  let segments = generate_segments(&parameters, 10 * 1000).unwrap();
  assert_eq!(
    segments,
    [MediaSegment {
      start: 0,
      end: 9999
    }]
  );
}

#[test]
fn duration() {
  use crate::duration::{Duration, DurationUnit};

  let parameters = MediaSplitterParameters {
    source_path: "fake_source.mxf".to_string(),
    output_parameter_name: crate::default_output_parameter_name(),
    number_of_segments: 1,
    duration: Some(Duration {
      value: 5,
      unit: DurationUnit::Percent,
    }),
    ..Default::default()
  };

  let segments = generate_segments(&parameters, 10 * 1000).unwrap();
  assert_eq!(segments, vec![MediaSegment { start: 0, end: 499 }]);
}

#[test]
fn max_duration() {
  use crate::duration::{Duration, DurationUnit};

  let parameters = MediaSplitterParameters {
    source_path: "fake_source.mxf".to_string(),
    output_parameter_name: crate::default_output_parameter_name(),
    number_of_segments: 1,
    max_duration: Some(Duration {
      value: 5,
      unit: DurationUnit::Second,
    }),
    ..Default::default()
  };

  let segments = generate_segments(&parameters, 10 * 1000).unwrap();
  assert_eq!(
    segments,
    vec![MediaSegment {
      start: 0,
      end: 4999
    }]
  );
}

#[test]
fn duration_max_duration() {
  use crate::duration::{Duration, DurationUnit};

  let parameters = MediaSplitterParameters {
    source_path: "fake_source.mxf".to_string(),
    output_parameter_name: crate::default_output_parameter_name(),
    number_of_segments: 2,
    duration: Some(Duration {
      value: 5,
      unit: DurationUnit::Percent,
    }),
    max_duration: Some(Duration {
      value: 5,
      unit: DurationUnit::Second,
    }),
    ..Default::default()
  };

  let segments = generate_segments(&parameters, 10 * 1000).unwrap();
  assert_eq!(
    segments,
    vec![
      MediaSegment { start: 0, end: 250 },
      MediaSegment {
        start: 251,
        end: 499
      }
    ]
  );

  let parameters = MediaSplitterParameters {
    source_path: "fake_source.mxf".to_string(),
    output_parameter_name: crate::default_output_parameter_name(),
    number_of_segments: 2,
    duration: Some(Duration {
      value: 60,
      unit: DurationUnit::Percent,
    }),
    max_duration: Some(Duration {
      value: 5,
      unit: DurationUnit::Second,
    }),
    ..Default::default()
  };

  let segments = generate_segments(&parameters, 10 * 1000).unwrap();
  assert_eq!(
    segments,
    vec![
      MediaSegment {
        start: 0,
        end: 2500
      },
      MediaSegment {
        start: 2501,
        end: 4999
      }
    ]
  );
}

#[test]
fn duration_at_the_end() {
  use crate::duration::{Duration, DurationPosition, DurationUnit};

  let parameters = MediaSplitterParameters {
    source_path: "fake_source.mxf".to_string(),
    output_parameter_name: crate::default_output_parameter_name(),
    number_of_segments: 2,
    duration: Some(Duration {
      value: 5,
      unit: DurationUnit::Percent,
    }),
    max_duration: Some(Duration {
      value: 5,
      unit: DurationUnit::Second,
    }),
    duration_position: DurationPosition::End,
    ..Default::default()
  };

  let segments = generate_segments(&parameters, 10 * 1000).unwrap();
  assert_eq!(
    segments,
    vec![
      MediaSegment {
        start: 9500,
        end: 9750
      },
      MediaSegment {
        start: 9751,
        end: 9999
      }
    ]
  );

  let parameters = MediaSplitterParameters {
    source_path: "fake_source.mxf".to_string(),
    output_parameter_name: crate::default_output_parameter_name(),
    number_of_segments: 2,
    duration: Some(Duration {
      value: 60,
      unit: DurationUnit::Percent,
    }),
    max_duration: Some(Duration {
      value: 5,
      unit: DurationUnit::Second,
    }),
    ..Default::default()
  };

  let segments = generate_segments(&parameters, 10 * 1000).unwrap();
  assert_eq!(
    segments,
    vec![
      MediaSegment {
        start: 0,
        end: 2500
      },
      MediaSegment {
        start: 2501,
        end: 4999
      }
    ]
  );
}
