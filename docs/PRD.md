# Timelapse Studio

## Product Requirements Document (PRD)

**Version:** 0.1  
**Status:** Initial specification — pending review  
**Product type:** Desktop application  
**Platform:** macOS first  
**Architecture:** Local-first

---

## 1. Product vision

Timelapse Studio is a desktop application designed to transform large collections of photographs into organized, high-quality timelapse videos.

It targets users working with long-term photographic sequences, particularly construction projects, architecture, infrastructure, outdoor environments and GoPro cameras.

The application should make it possible to import, analyze, clean, organize, sample, process and export thousands or hundreds of thousands of photographs through a controlled and understandable workflow.

The application must prioritize reliability, transparency, non-destructive operations and visual quality.

## 2. Product principles

1. **Local-first:** All primary operations run locally. No account or cloud service is required.

2. **Non-destructive:** Original photographs must never be modified by processing operations.

3. **Traceability:** File operations and processing steps must be recorded.

4. **Recoverability:** Interrupted operations should be detectable and recoverable whenever practical.

5. **Scalability:** The application must support collections of 10,000 to 100,000+ photographs.

6. **Transparency:** Users must be able to preview operations and understand their consequences.

7. **Modularity:** Processing capabilities must remain independent of the user interface.

8. **Progressive workflow:** Each processing stage should be independently usable and repeatable.

## 3. Target users

- Construction and architecture professionals documenting projects.

- Photographers working with long-term timelapse sequences.

- Users managing large GoPro image collections.

- Users who need to clean, organize and select images before video production.

## 4. Main workflow

The application is organized around six principal stages:

1. Import & Analyze

2. Clean Images

3. Consolidate

4. Sample & Select

5. Video Configuration

6. Generate & Export

Each stage should have its own configuration, preview and execution process.

Users should be able to return to previous stages without losing their work.

## 5. Functional requirements

### 5.1 Project management

- Create a new project.

- Open an existing project.

- Save project configuration and processing state.

- Persist project information in SQLite.

- Provide a derived, on-demand export record for portability and inspection.

- Detect missing or changed source files.

- Reopen a project without unnecessarily reprocessing unchanged files.

### 5.2 Import and inventory

The application must:

- Import photographs from a selected folder.

- Scan folders recursively.

- Support JPG, JPEG, PNG and TIFF initially.

- Keep original photographs in their existing locations unless the user explicitly chooses otherwise.

- Record paths, names, dimensions, file sizes and timestamps.

- Extract available EXIF metadata.

- Identify missing or suspicious metadata.

- Detect exact duplicates using file hashes.

- Generate thumbnails without loading entire collections into memory.

Supported metadata should include:

- EXIF DateTimeOriginal

- EXIF CreateDate

- Camera manufacturer and model

- Image dimensions

- Orientation

- File size

- GPS coordinates, when available

Timestamp priority:

1. EXIF DateTimeOriginal

2. EXIF CreateDate

3. Timestamp extracted from filename

4. Filesystem timestamp as configurable fallback

The application must identify timestamp uncertainty rather than silently assuming all timestamps are reliable.

### 5.3 Timeline analysis

Provide an interactive chronological representation of the imported collection.

Requirements:

- Sort photographs chronologically.

- Display image activity over time.

- Support aggregation by hour, day, week and month.

- Identify gaps and unusual activity.

- Highlight missing or identical timestamps.

- Allow users to inspect photographs from a selected period.

- Support zooming and navigation across long projects.

- Distinguish real gaps from potential metadata problems.

The application must not automatically interpret every gap as an error.

### 5.4 Image cleaning

Analyze image brightness using luminance-based measurements.

Available measurements should include:

- Average luminance.

- Median luminance.

- Percentage of pixels below a configurable threshold.

- Luminance histogram.

Users must be able to configure:

- Minimum average luminance.

- Maximum percentage of pixels below a selected threshold.

The interface must provide:

- A list or gallery of flagged photographs.

- Thumbnail previews.

- Individual image inspection.

- Manual acceptance or rejection.

- Comparison of selected images.

- Contact sheets.

- Preview of the consequences of cleaning.

Rejected images must be placed in a recoverable quarantine rather than permanently deleted.

### 5.5 Consolidation

Allow users to organize photographs from multiple source folders into a common working directory.

Examples of source directories:

- 157gopro/

- 158gopro/

- 159gopro/

Supported operations:

- Copy.

- Move.

- Preserve original directory structure.

- Generate a consolidated chronological sequence.

Naming strategies:

- Original filename, with collision handling.

- Source-folder prefix.

- Sequential numbering.

- Timestamp-based naming.

Requirements:

- Dry-run preview.

- Destination collision detection.

- Disk-space estimation.

- Operation history.

- Recovery where practical.

- No recompression of original photographs.

- Preservation of metadata.

### 5.6 Sampling and selection

Sampling must be based primarily on chronological timestamps rather than filesystem ordering.

Supported methods:

- Fixed time interval.

- Every Nth photograph.

- Uniform distribution over a selected period.

- Future possibility: adaptive sampling.

For fixed-interval sampling, users must be able to choose how the nearest photograph is selected:

- Closest to target timestamp.

- First photograph after target.

- Last photograph before target.

The same photograph must not be selected repeatedly for multiple targets unless explicitly supported as a separate option.

Preview information:

- Total photographs.

- Selected photographs.

- Reduction percentage.

- Estimated video duration.

- Effective sampling interval.

- Timeline representation.

- Contact sheet of selected images.

Excluded photographs must not be deleted automatically.

### 5.7 Video configuration

Configurable output:

**Frame rate**

- 12, 15, 24, 25, 30, 50 and 60 FPS.

- Custom value where technically supported.

**Playback multiplier**

- 0.5x

- 1x

- 2x

- 4x

- Custom

Playback speed and output FPS must remain separate concepts.

At 1x, each selected photograph corresponds to one output frame. At 2x, the resulting duration is half that of 1x. At 0.5x, duration is doubled.

Do not duplicate frames unnecessarily. Frame duplication or interpolation should be treated as separate processing choices.

**Resolution**

- Original dimensions.

- 4K UHD.

- 2.7K.

- 1080p.

- 720p.

- Custom.

Preserve aspect ratio. Provide fit, crop and letterbox options.

**Codecs**

- **AV1 (default)** — encoded via SVT-AV1 (BSD-3-Clause) in the bundled LGPL FFmpeg sidecar. Recommended output for all new timelines; modern players (macOS 14+, iOS 17+, Chrome, Firefox, VLC) already support it.
- H.264 — compatibility profile only, produced via macOS VideoToolbox hardware encoder (no software fallback; below the minimum supported macOS version the job is refused with a clear error rather than silently falling back).
- H.265/HEVC — compatibility profile for Apple devices that need 10-bit, also via VideoToolbox only.

Rationale and licence posture recorded in ADR-017 (`docs/DECISIONS.md`) and `docs/FFMPEG_AV1_ANALYSIS.md`. The default choice (AV1) reflects the timelapse workload where AV1's compression efficiency at high quality meaningfully reduces file size versus H.264 at similar visual fidelity, and SVT-AV1 is fast on Apple Silicon for a real encode.

**Container**

- MP4.

- MOV.

**Quality**

- Draft.

- Standard.

- High.

- Maximum.

- Advanced settings for CRF, bitrate, preset and profile.

Hardware encoding should be detected when available. Apple VideoToolbox is the initial macOS hardware-encoding target, with software fallback.

### 5.8 Image adjustments

Optional processing before video generation:

- Brightness.

- Exposure.

- Contrast.

- Saturation.

- White balance.

- Sharpening.

- Crop.

- Resize.

All adjustments must be non-destructive.

### 5.9 Flicker reduction

Flicker reduction is an advanced feature and an important part of the product.

It must:

- Analyze temporal luminance variations.

- Identify abrupt exposure inconsistencies.

- Estimate unwanted exposure changes.

- Apply smooth corrections over time.

- Preserve genuine lighting changes such as sunrise, sunset, clouds and day/night transitions.

- Avoid brightness pumping and excessive correction.

- Offer configurable smoothing strength.

- Provide before/after previews.

The algorithm must be validated against real GoPro sequences, not only synthetic examples.

### 5.10 Stabilization

Evaluate an optional stabilization module capable of:

- Detecting frame-to-frame movement.

- Estimating alignment.

- Correcting unwanted camera movement.

- Managing crop compensation.

Stabilization must remain independent of flicker reduction.

### 5.11 Contact sheets

Contact sheets are a core visual feature.

Users should be able to generate contact sheets for:

- All imported photographs.

- A selected period.

- Flagged images.

- Quarantined images.

- Selected sampling results.

- Final video sequence.

Include configurable thumbnail size, number of images per page, filename and timestamp. PDF or image export should be evaluated.

### 5.12 Export

- Generate a video using FFmpeg.

- Display progress.

- Support cancellation.

- Validate the resulting file.

- Display output metadata.

- Preview the generated video.

- Open the destination folder.

- Preserve export history in the `video_exports` table; the on-demand export record is generated from it and is not stored as state.

## 6. Non-functional requirements

### Performance

- Handle at least 100,000 photographs as a design target.

- Avoid loading all full-resolution images into memory.

- Use batch processing and parallel execution where appropriate.

- Cache thumbnails and analysis results.

- Keep the UI responsive during processing.

### Reliability

- Never overwrite originals.

- Avoid silent file replacement.

- Record file operations.

- Validate available disk space.

- Detect interrupted operations.

- Clean temporary files safely.

- Provide understandable error messages.

### Maintainability

- Separate UI and processing logic.

- Keep Rust modules focused.

- Use database migrations.

- Add automated tests for critical operations.

- Document architecture and important decisions.

### Localization

- Support the locales fr-CA and en, identified by BCP 47 tags (ADR-016).

- Manage UI strings as Fluent catalogs (`locales/fr/`, `locales/en/`), with all user-facing strings extracted from hardcoded text.

- fr-CA is a release blocker: a build that cannot be verified against the complete fr-CA catalog must not ship.

- Default to fr-CA when the OS locale is French or the region reads as Québec; otherwise default to en. The authoritative locale list is `_foundation/company/etzian.yaml` (`locales:`).

## 7. Out of scope for initial release

Unless explicitly approved later:

- Cloud synchronization.

- User accounts.

- Online storage.

- Mobile applications.

- Multi-user collaboration.

- Advanced RAW development.

- AI-based image enhancement.

- Automatic semantic image classification.

## 8. Initial success criteria

The first usable release should allow a user to:

1. Create and reopen a project.

2. Import a large photographic collection.

3. Extract and inspect metadata.

4. Visualize its chronological distribution.

5. Identify and quarantine dark images.

6. Select photographs using temporal sampling.

7. Generate a video with configurable FPS, resolution and codec.

8. Preview and validate the final export.

9. Resume work without losing project state.

## 9. Open questions

The following require evaluation before final implementation:

- Exact supported image formats and RAW roadmap.

- Best EXIF extraction library.

- FFmpeg packaging and licensing strategy.

- Optimal thumbnail and contact-sheet formats.

- Final flicker-reduction algorithm.

- Stabilization algorithm and dependencies.

- Whether a standalone CLI should be included later.

Resolved: project portability is the derived on-demand export record generated from SQLite (ADR-015); originals are never moved (ADR-006, ADR-007).
