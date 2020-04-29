use mcai_worker_sdk::parameter::media_segment::MediaSegment;

#[derive(Debug)]
pub enum SplitPolicy {
  SegmentDurationSeconds(u64),
  SegmentDurationMilliSeconds(u64),
  NumberOfSegments(u64),
}

impl SplitPolicy {
  pub fn split(
    self,
    media_duration: u64,
    segment_overlap: Option<i64>,
  ) -> Result<Vec<MediaSegment>, String> {
    let (segment_duration, number_of_segments) = self.get_parameters(media_duration);

    let overlap = segment_overlap.unwrap_or(0) as u64;

    let mut segments = Vec::with_capacity(number_of_segments as usize);
    let mut next_start = 0;
    let mut next_end = 0;

    for _ in 0..number_of_segments {
      next_end += segment_duration as u64 - 1;
      if next_end >= media_duration {
        next_end = media_duration - 1;
      }

      segments.push(MediaSegment::new(next_start, next_end));

      next_end += 1;
      if next_end < overlap {
        next_start = 0;
      } else {
        next_start = next_end - overlap;
      }
    }

    Ok(segments)
  }

  fn get_parameters(self, media_duration: u64) -> (u64, u64) {
    match self {
      SplitPolicy::SegmentDurationSeconds(mut segment_duration) => {
        if segment_duration == 0 {
          segment_duration = media_duration;
        } else {
          // convert to milliseconds
          segment_duration *= 1000;
        }
        let number_of_segments = (media_duration as f64 / segment_duration as f64).ceil() as u64;
        (segment_duration, number_of_segments)
      }
      SplitPolicy::SegmentDurationMilliSeconds(mut segment_duration) => {
        if segment_duration == 0 {
          segment_duration = media_duration;
        }
        let number_of_segments = (media_duration as f64 / segment_duration as f64).ceil() as u64;
        (segment_duration, number_of_segments)
      }
      SplitPolicy::NumberOfSegments(mut number_of_segments) => {
        if number_of_segments == 0 {
          number_of_segments = 1;
        }
        if number_of_segments >= media_duration {
          number_of_segments = media_duration;
        }
        let segment_duration = (media_duration as f64 / number_of_segments as f64).ceil() as u64;
        (segment_duration, number_of_segments)
      }
    }
  }
}

#[test]
pub fn test_split_media_range_based_on_segment_duration_in_milliseconds() {
  let segment_duration = 100;
  let media_duration = 100;
  let result =
    SplitPolicy::SegmentDurationMilliSeconds(segment_duration).split(media_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(1, segments.len());
  assert_eq!(segment_duration, segments[0].end - segments[0].start + 1);
}

#[test]
pub fn test_split_media_range_based_on_segment_duration_in_seconds() {
  let segment_duration = 10;
  let media_duration = 10000;
  let result = SplitPolicy::SegmentDurationSeconds(segment_duration).split(media_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(1, segments.len());
  assert_eq!(
    segment_duration * 1000,
    segments[0].end - segments[0].start + 1
  );
}

#[test]
pub fn test_split_media_range_based_on_number_of_segments() {
  let number_of_segments = 1;
  let media_duration = 100;
  let result = SplitPolicy::NumberOfSegments(number_of_segments).split(media_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(number_of_segments as usize, segments.len());
  assert_eq!(media_duration, segments[0].end - segments[0].start + 1);
}

#[test]
pub fn test_split_media_based_on_segment_duration_in_milliseconds() {
  let segment_duration = 10;
  let media_duration = 98;
  let result =
    SplitPolicy::SegmentDurationMilliSeconds(segment_duration).split(media_duration, None);
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
pub fn test_split_media_based_on_segment_duration_in_seconds() {
  let segment_duration = 1;
  let media_duration = 10000;
  let result = SplitPolicy::SegmentDurationSeconds(segment_duration).split(media_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(10, segments.len());
  let expected_segments = vec![
    (0, 999),
    (1000, 1999),
    (2000, 2999),
    (3000, 3999),
    (4000, 4999),
    (5000, 5999),
    (6000, 6999),
    (7000, 7999),
    (8000, 8999),
    (9000, media_duration - 1),
  ];
  for index in 0..segments.len() {
    assert_eq!(
      expected_segments[index],
      (segments[index].start, segments[index].end)
    );
  }
}

#[test]
pub fn test_split_media_based_on_number_of_segments() {
  let number_of_segments = 10;
  let media_duration = 98;
  let result = SplitPolicy::NumberOfSegments(number_of_segments).split(media_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(number_of_segments as usize, segments.len());
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
pub fn test_split_media_based_on_segment_duration_in_milliseconds_larger_than_media_duration() {
  let segment_duration = 200;
  let media_duration = 100;
  let result =
    SplitPolicy::SegmentDurationMilliSeconds(segment_duration).split(media_duration, None);
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
pub fn test_split_media_based_on_segment_duration_in_seconds_larger_than_media_duration() {
  let segment_duration = 2000;
  let media_duration = 1000;
  let result = SplitPolicy::SegmentDurationSeconds(segment_duration).split(media_duration, None);
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
pub fn test_split_media_based_on_number_of_segments_larger_than_media_duration() {
  let number_of_segments = 200;
  let media_duration = 100;
  let result = SplitPolicy::NumberOfSegments(number_of_segments).split(media_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(media_duration as usize, segments.len());
  for index in 0..segments.len() {
    assert_eq!(
      (index as u64, index as u64),
      (segments[index].start, segments[index].end)
    );
  }
}

#[test]
pub fn test_split_media_based_on_number_of_segments_equal_to_zero() {
  let number_of_segments = 0;
  let media_duration = 100;
  let result = SplitPolicy::NumberOfSegments(number_of_segments).split(media_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(1, segments.len());
}

#[test]
pub fn test_split_media_based_on_segment_duration_in_milliseconds_equal_to_zero() {
  let segment_duration = 0;
  let media_duration = 100;
  let result =
    SplitPolicy::SegmentDurationMilliSeconds(segment_duration).split(media_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(1, segments.len());
  assert_eq!(media_duration, segments[0].end - segments[0].start + 1);
}

#[test]
pub fn test_split_media_based_on_segment_duration_in_seconds_equal_to_zero() {
  let segment_duration = 0;
  let media_duration = 10000;
  let result = SplitPolicy::SegmentDurationSeconds(segment_duration).split(media_duration, None);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(1, segments.len());
  assert_eq!(media_duration, segments[0].end - segments[0].start + 1);
}

#[test]
pub fn test_split_media_based_on_segment_duration_in_milliseconds_with_overlap() {
  let segment_duration = 10;
  let segment_overlap = Some(4);
  let media_duration = 95;
  let result = SplitPolicy::SegmentDurationMilliSeconds(segment_duration)
    .split(media_duration, segment_overlap);
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
pub fn test_split_media_based_on_segment_duration_in_seconds_with_overlap() {
  let segment_duration = 1;
  let segment_overlap = Some(400);
  let media_duration = 9500;
  let result =
    SplitPolicy::SegmentDurationSeconds(segment_duration).split(media_duration, segment_overlap);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(10, segments.len());
  let expected_segments = vec![
    (0, 999),
    (600, 1999),
    (1600, 2999),
    (2600, 3999),
    (3600, 4999),
    (4600, 5999),
    (5600, 6999),
    (6600, 7999),
    (7600, 8999),
    (8600, media_duration - 1),
  ];
  for index in 0..segments.len() {
    assert_eq!(
      expected_segments[index],
      (segments[index].start, segments[index].end)
    );
  }
}

#[test]
pub fn test_split_media_based_on_number_of_segments_with_overlap() {
  let number_of_segments = 10;
  let segment_overlap = Some(4);
  let media_duration = 95;
  let result =
    SplitPolicy::NumberOfSegments(number_of_segments).split(media_duration, segment_overlap);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(number_of_segments as usize, segments.len());
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
pub fn test_split_media_based_on_segment_duration_in_milliseconds_with_overlap_larger_that_segment()
{
  let segment_duration = 10;
  let segment_overlap = Some(12);
  let result =
    SplitPolicy::SegmentDurationMilliSeconds(segment_duration).split(100, segment_overlap);
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

#[test]
pub fn test_split_media_based_on_segment_duration_in_seconds_with_overlap_larger_that_segment() {
  let segment_duration = 10;
  let segment_overlap = Some(12000);
  let media_duration = 100000;
  let result =
    SplitPolicy::SegmentDurationSeconds(segment_duration).split(media_duration, segment_overlap);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(10, segments.len());
  let expected_segments = vec![
    (0, 9999),
    (0, 19999),
    (8000, 29999),
    (18000, 39999),
    (28000, 49999),
    (38000, 59999),
    (48000, 69999),
    (58000, 79999),
    (68000, 89999),
    (78000, 99999),
  ];
  for index in 0..segments.len() {
    assert_eq!(
      expected_segments[index],
      (segments[index].start, segments[index].end)
    );
  }
}

#[test]
pub fn test_split_media_based_on_number_of_segments_with_overlap_larger_that_segment() {
  let number_of_segments = 10;
  let segment_overlap = Some(12);
  let result = SplitPolicy::NumberOfSegments(number_of_segments).split(100, segment_overlap);
  assert!(result.is_ok());
  let segments = result.unwrap();
  assert_eq!(number_of_segments as usize, segments.len());
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
