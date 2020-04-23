use amqp_worker::{
  job::{Job, JobResult, JobStatus},
  MessageError, ParametersContainer,
};
use lapin_futures::Channel;
use stainless_ffmpeg::format_context::FormatContext;

const SEGMENT_DURATION_PARAMETER: &'static str = "segment_duration";
const SEGMENT_OVERLAP_PARAMETER: &'static str = "segment_overlap";
const SOURCE_PATH_PARAMETER: &'static str = "source_path";

#[derive(Debug, Serialize, Deserialize)]
struct Segment {
  start: u64,
  end: u64,
}

impl Segment {
  fn new(start: u64, end: u64) -> Segment {
    Segment { start, end }
  }
}

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

  let segment_duration = job
    .get_integer_parameter(SEGMENT_DURATION_PARAMETER)
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

  let segments = split_media(media_duration, segment_duration, segment_overlap).map_err(|msg| {
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

  let duration_millisec =
    format_context
      .get_duration()
      .map(|duration| (duration * 1000.) as u64)
      .unwrap_or_else(|| 0);

  format_context.close_input();
  Ok(duration_millisec)
}

fn split_media(
  media_duration: u64,
  segment_duration: i64,
  segment_overlap: Option<i64>,
) -> Result<Vec<Segment>, String> {
  let number_of_segments = (media_duration as f64 / segment_duration as f64).ceil() as u64;
  let overlap = segment_overlap.unwrap_or(0) as u64;

  let mut segments = Vec::with_capacity(number_of_segments as usize);
  let mut next_start = 0;
  let mut next_end = 0;

  for _ in 0..number_of_segments {
    next_end += segment_duration as u64 - 1;
    if next_end >= media_duration {
      next_end = media_duration - 1;
    }

    segments.push(Segment::new(next_start, next_end));

    next_end += 1;
    if next_end < overlap {
      next_start = 0;
    } else {
      next_start = next_end - overlap;
    }
  }

  Ok(segments)
}

#[test]
pub fn test_split_media_range() {
  let segment_duration = 100;
  let result = split_media(100, segment_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(1, segments.len());

  let segment = &segments[0];
  let mut ms = segment.start;
  while ms <= segment.end {
    ms += 1;
  }
  assert_eq!(ms, segment_duration as u64);
}

#[test]
pub fn test_split_media() {
  let segment_duration = 10;
  let media_duration = 98;
  let result = split_media(media_duration, segment_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(10, segments.len());
  let expected_segments = vec![
    (0, 9),
    (10, 19),
    (20, 29),
    (30, 39),
    (40, 49),
    (50, 59),
    (60, 69),
    (70, 79),
    (80, 89),
    (90, media_duration - 1),
  ];
  for index in 0..segments.len() {
    assert_eq!(
      expected_segments[index],
      (segments[index].start, segments[index].end)
    );
  }
}

#[test]
pub fn test_split_media_with_segment_larger_than_duration() {
  let segment_duration = 200;
  let media_duration = 100;
  let result = split_media(media_duration, segment_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(1, segments.len());
  let expected_segments = vec![(0, media_duration - 1)];
  for index in 0..segments.len() {
    assert_eq!(
      expected_segments[index],
      (segments[index].start, segments[index].end)
    );
  }
}

#[test]
pub fn test_split_media_with_overlap() {
  let segment_duration = 10;
  let segment_overlap = Some(4);
  let media_duration = 95;
  let result = split_media(media_duration, segment_duration, segment_overlap);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(10, segments.len());
  let expected_segments = vec![
    (0, 9),
    (6, 19),
    (16, 29),
    (26, 39),
    (36, 49),
    (46, 59),
    (56, 69),
    (66, 79),
    (76, 89),
    (86, media_duration - 1),
  ];
  for index in 0..segments.len() {
    assert_eq!(
      expected_segments[index],
      (segments[index].start, segments[index].end)
    );
  }
}

#[test]
pub fn test_split_media_with_overlap_larger_that_segment() {
  let segment_duration = 10;
  let segment_overlap = Some(12);
  let result = split_media(100, segment_duration, segment_overlap);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(10, segments.len());
  let expected_segments = vec![
    (0, 9),
    (0, 19),
    (8, 29),
    (18, 39),
    (28, 49),
    (38, 59),
    (48, 69),
    (58, 79),
    (68, 89),
    (78, 99),
  ];
  for index in 0..segments.len() {
    assert_eq!(
      expected_segments[index],
      (segments[index].start, segments[index].end)
    );
  }
}
