# Timelapse Studio

## Architecture

**Version:** 0.1  
**Status:** Proposed architecture

---

## 1. Overview

Timelapse Studio is a local desktop application with a clear separation between presentation, application integration, processing and persistence.

```
┌───────────────────────────────────────┐
│             Svelte UI                 │
│                                       │
│ Workflow / Timeline / Gallery         │
│ Configuration / Preview / Progress   │
└───────────────────┬───────────────────┘
                    │
                    │ Tauri commands/events
                    ▼
┌───────────────────────────────────────┐
│            Tauri 2                    │
│       Application integration         │
└───────────────────┬───────────────────┘
                    │
                    ▼
┌───────────────────────────────────────┐
│           Rust application layer      │
│                                       │
│ Project / Import / Analysis           │
│ Sampling / File operations / Export   │
└──────────────┬───────────────┬────────┘
               │               │
               ▼               ▼
┌─────────────────────┐  ┌────────────────────┐
│ Processing services │  │ Persistence        │
│                     │  │                    │
│ EXIF / Image        │  │ SQLite             │
│ Timeline / Sampling │  │ Export record      │
│ Flicker / Stabilize │  │ Operation history  │
│ FFmpeg / Thumbnails │  │                    │
└─────────────────────┘  └────────────────────┘
```

## 2. Architectural rules

1. The frontend must not contain business-critical processing logic.

2. Rust services must not depend on Svelte or Tauri UI components.

3. SQLite is the authoritative source for project state.

4. Original photographs remain external to the project database.

5. All destructive-looking operations require explicit user intent.

6. Processing stages must be independently executable.

7. A stage must not silently invalidate previous work.

## 3. Main subsystems

### Project subsystem

Creates, opens and manages projects.

Responsibilities:

- Project identity.

- Project paths.

- Configuration.

- Database initialization.

- On-demand export-record generation (ADR-015).

- Project validation.

### Import subsystem

Discovers and registers photographs.

Responsibilities:

- Recursive scanning.

- File identification.

- Metadata extraction.

- Duplicate detection.

- Inventory persistence.

### Timeline subsystem

Creates a chronological representation of the image collection.

Responsibilities:

- Timestamp normalization.

- Chronological ordering.

- Gap analysis.

- Activity aggregation.

- Timeline queries.

### Image analysis subsystem

Analyzes image properties.

Responsibilities:

- Luminance calculations.

- Image quality measurements.

- Thumbnail creation.

- Image classification.

### File management subsystem

Performs controlled filesystem operations.

Responsibilities:

- Copy.

- Move.

- Naming.

- Collision detection.

- Quarantine.

- Recovery.

### Sampling subsystem

Creates a selected chronological sequence.

Responsibilities:

- Sampling strategies.

- Timestamp-based selection.

- Preview generation.

- Selection persistence.

### Video subsystem

Prepares and generates video output.

Responsibilities:

- Video configuration.

- Image adjustments.

- FFmpeg command construction.

- Encoding.

- Progress reporting.

- Output validation.

### Advanced processing subsystem

Independent modules for:

- Flicker reduction.

- Stabilization.

- Future image-processing features.

These modules should operate on derived processing data and must not modify source photographs.

## 4. Project directory

```
TimelapseProject/
├── project.db
├── thumbnails/
├── contact-sheets/
├── temp/
├── exports/          ← generated export record lands here, on demand
└── logs/
```

The project directory contains generated and administrative data. It does not need to contain the original photographs. Per ADR-015 there is no stored `manifest.json`: `project.db` (SQLite) is the sole source of truth, and the human-readable export record is a derived artifact written to `exports/` only when requested.

## 5. Data flow

```
Source directories
       │
       ▼
Import and inventory
       │
       ▼
Metadata extraction
       │
       ▼
Timeline analysis
       │
       ▼
Image analysis
       │
       ▼
Cleaning and classification
       │
       ▼
Optional consolidation
       │
       ▼
Temporal sampling
       │
       ▼
Optional image adjustments
       │
       ▼
Flicker reduction / stabilization
       │
       ▼
FFmpeg encoding
       │
       ▼
Video validation and export
```

This is a logical workflow, not necessarily a mandatory linear sequence. Users may revisit earlier stages.

## 6. Processing independence

Processing services should accept explicit inputs and return structured results.

For example, a sampling service should receive:

- A chronologically ordered set of eligible images.

- A sampling configuration.

- A selection policy.

It should return:

- Selected image identifiers.

- Target timestamps.

- Actual selected timestamps.

- Sampling statistics.

- Warnings.

It should not read frontend state directly.

## 7. Future extensibility

The architecture should permit:

- A command-line interface using the Rust engine.

- Additional image formats.

- Additional encoders.

- Additional sampling algorithms.

- Alternative flicker algorithms.

- Stabilization improvements.

- Future platform support.

Do not introduce abstractions solely for hypothetical future requirements. Keep the initial implementation understandable and testable.
