# Timelapse Studio

## Processing Pipeline

**Version:** 0.1  
**Status:** Initial workflow specification

---

## 1. General principles

The processing pipeline transforms a source collection into a validated video while preserving original photographs.

Each stage must:

- Have explicit inputs and outputs.

- Be independently executable where practical.

- Record its configuration.

- Report warnings and errors.

- Support preview before significant file operations.

- Avoid modifying source images.

- Preserve enough information to reproduce its results.

## 2. Stage 1 — Import and inventory

### Input

One or more selected source directories.

### Processing

1. Validate selected directories.

2. Scan recursively.

3. Identify supported image files.

4. Extract file metadata.

5. Extract available EXIF metadata.

6. Parse timestamps from filenames where possible.

7. Calculate file hashes for exact duplicate detection.

8. Register images in SQLite.

9. Generate thumbnails.

10. Produce an import report.

### Output

A complete image inventory.

### Important conditions

- Handle duplicate filenames in different folders.

- Handle GoPro numbering resets.

- Detect missing timestamps.

- Detect identical timestamps.

- Avoid assuming filesystem timestamps are capture timestamps.

- Handle unreadable and corrupt files.

## 3. Stage 2 — Metadata normalization and timeline

### Input

Imported inventory.

### Processing

1. Select the best available timestamp according to the defined priority.

2. Preserve original timestamp values.

3. Record the selected timestamp source.

4. Flag uncertain timestamps.

5. Normalize timestamps for chronological queries.

6. Sort images.

7. Calculate temporal intervals.

8. Identify unusually long gaps.

9. Generate timeline aggregation data.

### Output

A chronological representation of the collection.

### Important conditions

- Handle timezone ambiguity.

- Do not silently correct camera-clock errors.

- Allow manual timeline adjustments in a future iteration.

- Distinguish real capture gaps from metadata inconsistencies.

## 4. Stage 3 — Brightness analysis and cleaning

### Input

Imported and chronologically indexed images.

### Processing

1. Decode images at an appropriate analysis resolution.

2. Calculate luminance.

3. Calculate average and median luminance.

4. Calculate the percentage of dark pixels.

5. Compare measurements with user thresholds.

6. Flag images for review.

7. Generate review thumbnails and contact sheets.

### Output

A classified image collection.

### Cleaning policy

A flagged image is not automatically deleted.

The user must explicitly confirm quarantine operations.

### Quarantine

- Record original path.

- Record destination path.

- Record operation status.

- Preserve recovery information.

- Verify successful movement.

- Allow restoration.

## 5. Stage 4 — Consolidation

### Input

Selected source images and destination configuration.

### Processing

1. Preview destination structure.

2. Determine filenames.

3. Detect collisions.

4. Estimate required disk space.

5. Generate a dry-run report.

6. Request user confirmation.

7. Execute copy or move operations.

8. Verify results.

9. Record operation history.

### Output

An optional consolidated working collection.

### Safety

Original photographs must not be overwritten.

A move operation must be traceable and recoverable where possible.

## 6. Stage 5 — Temporal sampling

### Input

Eligible images in chronological order.

### Supported strategies

**Fixed interval**

Generate target timestamps at a configured interval.

**Every Nth image**

Select images according to chronological sequence order.

**Uniform distribution**

Select a configured number of images distributed across the selected period.

### Selection rules

- Select by timestamps where applicable.

- Support closest, first-after and last-before target policies.

- Avoid duplicate selection of the same image.

- Report unmatched targets.

- Handle missing timestamps explicitly.

### Output

A sampling run and ordered selection.

### Preview

Display:

- Original count.

- Selected count.

- Reduction percentage.

- Estimated duration.

- Effective interval.

- Timeline.

- Contact sheet.

## 7. Stage 6 — Image adjustments

Optional image transformations should operate on derived image data.

Possible adjustments:

- Brightness.

- Exposure.

- Contrast.

- Saturation.

- White balance.

- Sharpening.

- Crop.

- Resize.

The application must preserve the original photographs.

The exact order of adjustments must be documented and tested.

## 8. Stage 7 — Flicker reduction

### Input

A chronological selected sequence.

### Analysis

- Calculate temporal luminance characteristics.

- Compare adjacent frames.

- Identify abrupt and potentially unwanted exposure changes.

- Estimate trends and local variations.

- Distinguish likely camera exposure fluctuations from real lighting changes.

### Correction

- Apply relative exposure corrections.

- Smooth corrections over time.

- Offer configurable smoothing strength.

- Preserve genuine illumination changes.

### Output

A derived corrected sequence or equivalent processing instructions.

### Validation

- Before/after image comparison.

- Preview video.

- Test against real GoPro datasets.

- Inspect sunrise, sunset, clouds and day/night transitions.

- Detect brightness pumping and loss of contrast.

Do not adopt a simple global brightness filter as the final solution without validation.

## 9. Stage 8 — Stabilization

Evaluate frame alignment and motion estimation.

Potential operations:

- Feature detection.

- Motion estimation.

- Frame alignment.

- Crop compensation.

Stabilization must be independent of flicker reduction.

## 10. Stage 9 — Video generation

### Input

Final ordered sequence and video configuration.

### Processing

1. Validate selected images.

2. Validate configuration.

3. Check FFmpeg availability and capabilities.

4. Check output directory and available space.

5. Prepare controlled FFmpeg inputs.

6. Execute encoding.

7. Capture progress and diagnostics.

8. Support cancellation.

9. Validate generated output.

### Video duration

At 1x playback:

`duration = selected_image_count / output_fps`

Playback multipliers modify the resulting duration according to the selected speed.

Frame duplication and interpolation must be separate explicit options.

### FFmpeg safety

- Do not construct shell commands from uncontrolled strings.

- Use explicit argument lists.

- Avoid wildcard-based image selection.

- Handle filenames safely.

- Capture stderr and exit status.

- Validate output using available FFmpeg inspection tools.

## 11. Stage 10 — Export and validation

Validate:

- File existence.

- File size.

- Container.

- Codec.

- Resolution.

- Frame rate.

- Duration.

- Successful decoding.

Store export configuration and result in SQLite.

## 12. Recovery strategy

Every significant filesystem operation must record enough information to determine:

- What was intended.

- What was completed.

- What remains.

- Whether the operation can be safely resumed.

- Whether restoration is possible.

A database transaction alone cannot guarantee atomicity across filesystem operations. Use explicit operation states and reconciliation after interruption.

## 13. Pipeline reproducibility

Store:

- Processing configuration.

- Algorithm version.

- Input image identifiers.

- Output references.

- Execution status.

- Warnings.

- Error details.

The goal is to make processing results explainable and repeatable.
