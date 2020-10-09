use mcai_worker_sdk::JsonSchema;

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct Duration {
  #[serde(default = "default_duration")]
  pub value: u64,
  #[serde(default = "DurationUnit::default")]
  pub unit: DurationUnit,
}

impl Default for Duration {
  fn default() -> Duration {
    Duration {
      value: default_duration(),
      unit: DurationUnit::default(),
    }
  }
}

impl Duration {
  pub fn to_millis(&self, media_duration: u64) -> u64 {
    match self.unit {
      DurationUnit::Millisecond => self.value,
      DurationUnit::Second => self.value * 1000,
      DurationUnit::Percent => media_duration * self.value / 100,
    }
  }
}

fn default_duration() -> u64 {
  1
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub enum DurationUnit {
  #[serde(rename = "millisecond")]
  Millisecond,
  #[serde(rename = "second")]
  Second,
  #[serde(rename = "percent")]
  Percent,
}

impl Default for DurationUnit {
  fn default() -> DurationUnit {
    DurationUnit::Second
  }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub enum DurationPosition {
  #[serde(rename = "start")]
  Start,
  #[serde(rename = "end")]
  End,
}

impl Default for DurationPosition {
  fn default() -> DurationPosition {
    DurationPosition::Start
  }
}

#[test]
fn duration_checks() {
  let media_duration = 666;
  let duration = Duration::default();

  let ms_duration: u64 = duration.to_millis(media_duration);
  assert_eq!(ms_duration, 1000);

  let duration = Duration {
    value: 10,
    unit: DurationUnit::Second,
  };

  let ms_duration: u64 = duration.to_millis(media_duration);
  assert_eq!(ms_duration, 10000);

  let duration = Duration {
    value: 10,
    unit: DurationUnit::Millisecond,
  };

  let ms_duration: u64 = duration.to_millis(media_duration);
  assert_eq!(ms_duration, 10);

  let duration = Duration {
    value: 5,
    unit: DurationUnit::Percent,
  };

  let ms_duration: u64 = duration.to_millis(media_duration);
  assert_eq!(ms_duration, 33);
}
