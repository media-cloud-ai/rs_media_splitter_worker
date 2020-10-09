# rs_media_splitter_worker

Split a media into multiple segments.
It can also select a part of the content.

## Examples


### Split into 100 parts
```bash
export SOURCE_PATH=test_file.ext # replace with a real path
RUST_LOG=debug SOURCE_ORDERS=examples/split_100_segments.json cargo run
```

### Process 10% of the content, with maximum 5 seconds
```bash
export SOURCE_PATH=test_file.ext # replace with a real path
RUST_LOG=debug SOURCE_ORDERS=examples/process_10_percent_max_5_seconds.json cargo run
```

### Process 10% of the content, with maximum 5 seconds at the end
```bash
export SOURCE_PATH=test_file.ext # replace with a real path
RUST_LOG=debug SOURCE_ORDERS=examples/process_end_of_content.json cargo run
```
