## App-specific strings. Shared ones (actions, errors, settings, legal, consent)
## come from _foundation/i18n/common and live beside this file.

-app-name = Timelapse Studio
app-name = { -app-name }
app-tagline = Turn large photo collections into high-quality timelapse videos, locally.
company-name = Etzian inc.

nav-home = Home
nav-settings = Settings

home-title = Import collection
home-intro = Pick the folder that contains your photographs. Timelapse Studio scans it, hashes the supported formats (JPG, PNG, TIFF), looks for content duplicates and records everything in a project database on this device.
home-import-action = Choose a folder to import
home-importing = Importing…
home-stat-scanned = Files scanned
home-stat-hashed = Files hashed
home-stat-new = New entries
home-stat-unchanged = Already present
home-stat-duplicates = Duplicate files
home-skipped-unreadable = { $count } file(s) could not be read and were imported without a hash.
home-duplicate-groups-summary = Duplicate groups found in this import
home-integrity-missing = { $count } inventory entry(ies) has/have no file on disk
home-integrity-dups = { $count } distinct duplicate group(s) in the inventory
home-root = Source folder: { $path }

settings-section-privacy = Privacy
settings-privacy-description = Everything stays on this device. Nothing is sent anywhere.
erase-item-settings = Your language, theme and density choices

about-endorsed-by = by

## Phase 2 — timeline
timeline-dating = Reading capture dates…
timeline-backfill-result = { $resolved ->
    [one] One image was dated in this pass
   *[other] { $resolved } images were dated in this pass
}{ $unresolved ->
    [0] .
    [one] ; one could not be dated.
   *[other] ; { $unresolved } could not be dated.
}
timeline-title = Timeline
timeline-summary-resolved = Dated images
timeline-summary-unresolved = Undated images
timeline-summary-low-confidence = Low confidence
timeline-summary-tz-assumed = Time zone not recorded
timeline-by-source = Where the dates come from
timeline-source-exif-original = Camera clock (EXIF, original)
timeline-source-exif-digitized = Camera clock (EXIF, digitized)
timeline-source-filename = File name
timeline-source-filesystem = File modification date
timeline-confidence-high = High confidence
timeline-confidence-medium = Medium confidence
timeline-confidence-low = Low confidence
timeline-tz-assumed = Zone not recorded
timeline-unresolved = No date
timeline-dates-utc = Dates are shown in UTC. When a camera did not record its time zone, its clock is read as UTC; no date is ever corrected.
timeline-empty-title = No dated images yet
timeline-empty-description = Once an imported photograph has a capture date (camera, file name or file date), it appears here in chronological order.
timeline-distribution = Distribution over the period
timeline-granularity-hour = By hour
timeline-granularity-day = By day
timeline-granularity-month = By month
timeline-gaps-title = { $count ->
    [one] One unusually long gap
   *[other] { $count } unusually long gaps
}
timeline-gaps-none = No unusually long gap (threshold: { $threshold }).
timeline-gap = { $from } → { $to } ({ $duration })
timeline-gap-source-changed = The dates on either side come from different sources: this may be a metadata inconsistency rather than a pause in shooting.
timeline-list-title = Images in chronological order
timeline-list-truncated = Showing { $shown } of { $total } images.
timeline-show-more = Show more
timeline-loading = Building the timeline…
timeline-retry = Try again

## Errors (rendered via i18n/errors.ts → t(`error-${kind}`, args)).
error-import-root-unreadable = The folder “{ $path }” could not be read.
error-image-unreadable = A photograph at “{ $path }” could not be hashed.
error-database = The project database could not be opened or written at “{ $path }”.
error-store-transaction = Saving the import into the project database failed.
