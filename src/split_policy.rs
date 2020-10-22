use crate::MediaSplitterParameters;
use mcai_worker_sdk::{info, parameter::media_segment::MediaSegment};
use std::cmp::min;

#[derive(Debug)]
pub struct SplitPolicy {
  number_of_segments: u64,
  min_segment_duration: Option<u64>,
}

impl Default for SplitPolicy {
  fn default() -> Self {
    SplitPolicy {
      number_of_segments: 1,
      min_segment_duration: None,
    }
  }
}

impl SplitPolicy {
  pub fn new(parameters: &MediaSplitterParameters, media_duration: u64) -> Self {
    let min_segment_duration: Option<u64> = parameters
      .min_segment_duration
      .clone()
      .map(|duration| duration.to_millis(media_duration));

    SplitPolicy {
      number_of_segments: parameters.number_of_segments,
      min_segment_duration,
    }
  }

  pub fn split(
    self,
    media_duration: u64,
    start_offset: u64,
    segment_overlap: Option<u64>,
  ) -> Vec<MediaSegment> {
    let mut number_of_segments = self.number_of_segments;

    if let Some(min_segment_duration) = &self.min_segment_duration {
      if min_segment_duration > &0 {
        let max_number_of_segments = media_duration / min_segment_duration;
        number_of_segments = min(number_of_segments, max_number_of_segments);
      }
    }

    number_of_segments = min(number_of_segments, media_duration);

    info!("Number of segments: {}", number_of_segments);

    let overlap = segment_overlap.unwrap_or(0);

    let mut segments = Vec::with_capacity(number_of_segments as usize);
    let mut next_start = 0;
    let mut next_end = 0;

    for segment_index in 0..number_of_segments {
      let next_segment_duration = if media_duration - next_end == number_of_segments {
        1
      } else {
        let remaining_duration = media_duration - next_end;
        let remaining_segments = number_of_segments - segment_index;
        (remaining_duration as f64 / remaining_segments as f64) as u64
      };

      next_end += next_segment_duration as u64;
      if next_end >= media_duration {
        next_end = media_duration;
      }

      segments.push(MediaSegment::new(
        next_start + start_offset,
        next_end + start_offset,
      ));

      if next_end >= media_duration {
        break;
      }

      next_start = if next_end < overlap {
        0
      } else {
        next_end - overlap
      };
    }

    segments
  }
}

#[test]
pub fn empty_parameters() {
  let media_duration = 100;
  let split_policy = SplitPolicy::default();

  let segments = split_policy.split(media_duration, 0, None);

  assert_eq!(1, segments.len());
  assert_eq!(segments, vec![MediaSegment { start: 0, end: 100 }]);
}

#[test]
pub fn segments() {
  let media_duration = 100;
  let split_policy = SplitPolicy {
    number_of_segments: 3,
    min_segment_duration: None,
  };

  let segments = split_policy.split(media_duration, 0, None);

  assert_eq!(3, segments.len());
  assert_eq!(
    segments,
    vec![
      MediaSegment { start: 0, end: 33 },
      MediaSegment { start: 33, end: 66 },
      MediaSegment {
        start: 66,
        end: 100
      }
    ]
  );
}

#[test]
pub fn number_of_segments_upper_than_duration() {
  let media_duration = 10;
  let split_policy = SplitPolicy {
    number_of_segments: 100,
    min_segment_duration: None,
  };

  let segments = split_policy.split(media_duration, 0, None);

  assert_eq!(10, segments.len());
  assert_eq!(
    segments,
    vec![
      MediaSegment { start: 0, end: 1 },
      MediaSegment { start: 1, end: 2 },
      MediaSegment { start: 2, end: 3 },
      MediaSegment { start: 3, end: 4 },
      MediaSegment { start: 4, end: 5 },
      MediaSegment { start: 5, end: 6 },
      MediaSegment { start: 6, end: 7 },
      MediaSegment { start: 7, end: 8 },
      MediaSegment { start: 8, end: 9 },
      MediaSegment { start: 9, end: 10 },
    ]
  );

  let media_duration = 10;
  let split_policy = SplitPolicy {
    number_of_segments: 10,
    min_segment_duration: None,
  };

  let segments = split_policy.split(media_duration, 0, None);

  assert_eq!(10, segments.len());
  assert_eq!(
    segments,
    vec![
      MediaSegment { start: 0, end: 1 },
      MediaSegment { start: 1, end: 2 },
      MediaSegment { start: 2, end: 3 },
      MediaSegment { start: 3, end: 4 },
      MediaSegment { start: 4, end: 5 },
      MediaSegment { start: 5, end: 6 },
      MediaSegment { start: 6, end: 7 },
      MediaSegment { start: 7, end: 8 },
      MediaSegment { start: 8, end: 9 },
      MediaSegment { start: 9, end: 10 },
    ]
  );

  let media_duration = 11;
  let split_policy = SplitPolicy {
    number_of_segments: 6,
    min_segment_duration: None,
  };

  let segments = split_policy.split(media_duration, 0, None);

  assert_eq!(6, segments.len());
  assert_eq!(
    segments,
    vec![
      MediaSegment { start: 0, end: 1 },
      MediaSegment { start: 1, end: 3 },
      MediaSegment { start: 3, end: 5 },
      MediaSegment { start: 5, end: 6 },
      MediaSegment { start: 6, end: 8 },
      MediaSegment { start: 8, end: 11 },
    ]
  );
}

#[test]
fn min_segment_duration() {
  let media_duration = 100;
  let split_policy = SplitPolicy {
    number_of_segments: 1,
    min_segment_duration: Some(40),
  };

  let segments = split_policy.split(media_duration, 0, None);

  assert_eq!(1, segments.len());
  assert_eq!(segments, vec![MediaSegment { start: 0, end: 100 }]);
}

#[test]
fn min_segment_duration_with_segments() {
  let media_duration = 100;
  let split_policy = SplitPolicy {
    number_of_segments: 5,
    min_segment_duration: Some(10),
  };

  let segments = split_policy.split(media_duration, 0, None);

  assert_eq!(5, segments.len());
  assert_eq!(
    segments,
    vec![
      MediaSegment { start: 0, end: 20 },
      MediaSegment { start: 20, end: 40 },
      MediaSegment { start: 40, end: 60 },
      MediaSegment { start: 60, end: 80 },
      MediaSegment {
        start: 80,
        end: 100
      }
    ]
  );
}

#[test]
fn overlap() {
  let media_duration = 100;
  let split_policy = SplitPolicy {
    number_of_segments: 5,
    min_segment_duration: None,
  };

  let segments = split_policy.split(media_duration, 0, Some(5));

  assert_eq!(5, segments.len());
  assert_eq!(
    segments,
    vec![
      MediaSegment { start: 0, end: 20 },
      MediaSegment { start: 15, end: 40 },
      MediaSegment { start: 35, end: 60 },
      MediaSegment { start: 55, end: 80 },
      MediaSegment {
        start: 75,
        end: 100
      }
    ]
  );
}

#[test]
fn offset() {
  let media_duration = 70;
  let split_policy = SplitPolicy {
    number_of_segments: 3,
    min_segment_duration: None,
  };

  let segments = split_policy.split(media_duration, 30, None);

  assert_eq!(3, segments.len());
  assert_eq!(
    segments,
    vec![
      MediaSegment { start: 30, end: 53 },
      MediaSegment { start: 53, end: 76 },
      MediaSegment {
        start: 76,
        end: 100
      }
    ]
  );
}
