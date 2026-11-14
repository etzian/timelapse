## App-specific strings. Shared ones (actions, errors, settings, legal, consent)
## come from _foundation/i18n/common and live beside this file.

-app-name = Timelapse Studio
app-name = { -app-name }
app-tagline = Turn large photo collections into high-quality timelapse videos, locally.
company-name = Etzian inc.

nav-home = Home
nav-cleaning = Cleaning
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

## Phase 3 — cleaning
clean-title = Cleaning
clean-intro = Timelapse Studio measures the brightness of each photograph and flags the ones that look too dark, or that Lightroom marked as rejected. Nothing is removed automatically: you decide, and a rejected photo only moves to the project's quarantine when you confirm.
clean-no-project-title = No collection open
clean-no-project-description = Import a folder first; its photographs can then be analysed and cleaned here.
clean-go-import = Go to import
clean-thresholds-title = Thresholds
clean-threshold-min-mean = Minimum average brightness (0–255)
clean-threshold-dark-level = Dark pixel level (0–255)
clean-threshold-max-dark = Maximum share of dark pixels (%)
clean-threshold-xmp = Flag photos rejected in Lightroom
clean-thresholds-invalid = Enter whole or decimal numbers within the ranges shown (the dark level must be a whole number).
clean-thresholds-note = Changing a threshold re-evaluates the photos you have not decided on; your own decisions are kept.
clean-apply = Apply thresholds
clean-applying = Applying…
clean-analysis-title = Analysis
clean-analyze = Analyse photographs
clean-analyzing = Analysing…
clean-progress = { $done } of { $total }
clean-loading = Loading…
clean-no-preview = No preview
clean-retry = Try again
clean-dismiss = Dismiss
clean-status-eligible = Kept
clean-status-flagged = To review
clean-status-rejected = Rejected
clean-status-quarantined = In quarantine
clean-status-unanalyzed = Not analysed
clean-filter-label = Show
clean-filter-flagged = To review
clean-filter-rejected = Rejected
clean-filter-quarantined = Quarantine
clean-filter-all = All
clean-filter-empty-flagged = Nothing to review
clean-filter-empty-rejected = No rejected photographs
clean-filter-empty-quarantined = The quarantine is empty
clean-filter-empty-all = No photographs
clean-filter-empty-description = Other photographs are under the other tabs.
clean-show-all = Show all
clean-empty-title = No photographs in this collection
clean-empty-description = Import a folder that contains JPG, PNG or TIFF files.
clean-unanalyzed-title = Not analysed yet
clean-unanalyzed-description = Analyse the collection to measure the brightness of each photograph.
clean-inspect = Inspect { $path }
clean-user-decision = Your decision
clean-metrics = Average { $mean } · { $dark } dark
clean-accept = Keep
clean-reject = Reject
clean-clear = Undo decision
clean-restore = Restore
clean-shown = Showing { $shown } of { $total }.
clean-show-more = Show more
clean-quarantine-action = { $count ->
    [one] Move one rejected photo to quarantine…
   *[other] Move { $count } rejected photos to quarantine…
}
clean-confirm-title = Move to quarantine?
clean-confirm-body = { $count ->
    [one] One photograph will be moved to { $path }.
   *[other] { $count } photographs will be moved to { $path }.
}
clean-confirm-safety = Each file is checked before and after the move and can be restored at any time from the Quarantine tab. Nothing is deleted.
clean-confirm-action = { $count ->
    [one] Move one photo
   *[other] Move { $count } photos
}
clean-cancel = Cancel
clean-moving = Moving…
clean-quarantined-result = { $ok ->
    [one] One photograph moved to quarantine
   *[other] { $ok } photographs moved to quarantine
}{ $failed ->
    [0] .
    [one] ; one could not be moved:
   *[other] ; { $failed } could not be moved:
}
clean-restored-result = { $ok ->
    [one] One photograph restored
   *[other] { $ok } photographs restored
}{ $failed ->
    [0] .
    [one] ; one could not be restored:
   *[other] ; { $failed } could not be restored:
}
clean-detail-status = Status
clean-detail-mean = Average brightness
clean-detail-median = Median brightness
clean-detail-dark = Dark pixels
clean-detail-xmp = Lightroom rating
clean-detail-location = In quarantine at
clean-detail-histogram = Brightness histogram
clean-reason-mean-below = Average brightness { $mean } is below { $min }
clean-reason-too-many-dark-pixels = { $fraction } of the pixels are dark (limit { $max })
clean-reason-xmp-rejected = Rejected in Lightroom
clean-reason-analysis-failed = Could not be analysed: { $failure }
clean-failure-unreadable = the file could not be read
clean-failure-undecodable = the file is not a readable image
files-failure-unknown-image = this photograph is no longer in the project
files-failure-not-rejected = only rejected photographs can be quarantined
files-failure-not-quarantined = this photograph is not in quarantine
files-failure-unsafe-path = its path could not be placed safely in the quarantine
files-failure-source-unreadable = the file could not be read
files-failure-changed-since-import = the file has changed since it was imported, so it was not moved
files-failure-destination-exists = a file already exists at the destination; nothing was overwritten
files-failure-move-failed = the file could not be moved; it is still where it was
files-failure-source-in-use = the file is in use or read-only; it is still where it was
files-failure-record-failed = the file was moved but not yet recorded; the app settles this automatically
files-failure-verify-failed = the copy did not match the original and was discarded; the original is still where it was
files-failure-interrupted = the move was interrupted and undone; the file is still where it was
files-failure-inconsistent = the move was interrupted and left an unexpected state; no file was touched
files-failure-other = the operation failed

## Errors (rendered via i18n/errors.ts → t(`error-${kind}`, args)).
error-import-root-unreadable = The folder “{ $path }” could not be read.
error-image-unreadable = The photograph at “{ $path }” could not be read.
error-database = The project database could not be opened or written at “{ $path }”.
error-store-transaction = Saving to the project database failed.
error-image-undecodable = The photograph at “{ $path }” is not a readable image.
error-unknown-image = This photograph is no longer in the project.
