## App-specific strings. Shared ones (actions, errors, settings, legal, consent)
## come from _foundation/i18n/common and live beside this file.

-app-name = Timelapse Studio
app-name = { -app-name }
app-tagline = Turn large photo collections into high-quality timelapse videos, locally.
company-name = Etzian inc.

nav-home = Home
nav-cleaning = Cleaning
nav-organize = Organize
nav-sampling = Sampling
nav-adjust = Adjust
nav-export = Export
nav-sheets = Contact sheets
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
home-hidden-removed = { $count ->
    [one] { $count } hidden macOS file (a “._” copy of a photo’s metadata) imported earlier was removed from the project. Your photographs are not affected.
   *[other] { $count } hidden macOS files (“._” copies of photos’ metadata) imported earlier were removed from the project. Your photographs are not affected.
}
home-skipped-unreadable = { $count } file(s) could not be read and were imported without a hash.
home-duplicate-groups-summary = Duplicate groups found in this import
home-integrity-missing = { $count } inventory entry(ies) has/have no file on disk
home-integrity-dups = { $count } distinct duplicate group(s) in the inventory
home-root = Source folder: { $path }

settings-section-privacy = Privacy
settings-privacy-description = Everything stays on this device. Nothing is sent anywhere.
erase-item-settings = Your language, theme and density choices
erase-section = Choose what to erase
erase-section-description = Then use Erase all data below. Your original photographs are never touched, except the rejected ones in the quarantine if you choose it.
erase-choice-settings = App settings
erase-choice-project = Project data
erase-choice-project-hint = The hidden “.timelapse” folder of { $folder } ({ $size }): decisions, selections, adjustment runs, thumbnails, temporary files and the undo history.
erase-choice-exports = Exports
erase-choice-exports-hint = { $count ->
    [one] One file ({ $size }) in the “Timelapse exports” folder: videos, their records and contact sheets.
   *[other] { $count } files ({ $size }) in the “Timelapse exports” folder: videos, their records and contact sheets.
}
erase-choice-copies = Copied photographs
erase-choice-copies-hint = { $count ->
    [one] One copy ({ $size }) made by consolidation in { $folders }. A copy you changed is kept.
   *[other] { $count } copies ({ $size }) made by consolidation in { $folders }. A copy you changed is kept.
}
erase-choice-copies-moved = { $count ->
    [one] One photograph moved by consolidation is never erased: it is the only copy.
   *[other] { $count } photographs moved by consolidation are never erased: they are the only copy.
}
erase-choice-quarantine = Quarantine
erase-choice-quarantine-hint = { $count ->
    [one] One rejected original photograph ({ $size }) in the “Timelapse quarantine” folder. It is deleted permanently.
   *[other] { $count } rejected original photographs ({ $size }) in the “Timelapse quarantine” folder. They are deleted permanently.
}
erase-choice-empty = Nothing to erase.
erase-no-project = Open a source folder on Home to also erase its project data, exports, copies or quarantine.
erase-nothing-chosen = Nothing: choose at least one item above.
erase-item-project = The project data of { $folder } ({ $size })
erase-item-exports = { $count ->
    [one] The exports: one file ({ $size })
   *[other] The exports: { $count } files ({ $size })
}
erase-item-copies = { $count ->
    [one] One photograph copied by consolidation ({ $size })
   *[other] { $count } photographs copied by consolidation ({ $size })
}
erase-item-quarantine = { $count ->
    [one] One rejected original photograph in the quarantine ({ $size })
   *[other] { $count } rejected original photographs in the quarantine ({ $size })
}
erase-done-copies-kept = { $count ->
    [one] One copy was kept: it changed, went missing or is now an original.
   *[other] { $count } copies were kept: they changed, went missing or are now originals.
}
error-erase = { $reason ->
    [busy] Work is running on this folder. Wait for it to finish or stop it, then erase.
    [quarantine-inside] Rejected originals are still inside the project data. Choose the quarantine too, or restore them first.
    [remove-failed] A file could not be removed; it may be open in another app. Close it and try again.
   *[other] Erasing did not complete.
}

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
timeline-granularity-week = By week
timeline-granularity-month = By month
# A week bucket's label; $date is the Monday that starts it.
timeline-week-of = Week of { $date }
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
clean-stop = Stop
clean-stopping = Stopping…
clean-stopped = { $count ->
    [one] Analysis stopped. What was analysed is kept; { $count } photograph is left. Analyse again to continue.
   *[other] Analysis stopped. What was analysed is kept; { $count } photographs are left. Analyse again to continue.
}
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
clean-bulk-label = Decide for several photographs
clean-select-photo = Select { $path }
clean-select-all = Select all { $count }
clean-select-none = Clear selection
clean-selected = { $count ->
    [one] { $count } photograph selected
   *[other] { $count } photographs selected
}
clean-selected-beyond = { $count ->
    [one] including { $count } not shown below
   *[other] including { $count } not shown below
}
clean-bulk-accept = Keep selected
clean-bulk-reject = Reject selected
clean-bulk-clear = Clear their decisions
clean-bulk-accept-title = { $count ->
    [one] Keep { $count } photograph?
   *[other] Keep { $count } photographs?
}
clean-bulk-accept-body = { $count ->
    [one] It stays available to sampling and the video, whatever the analysis proposed.
   *[other] They stay available to sampling and the video, whatever the analysis proposed.
}
clean-bulk-accept-action = { $count ->
    [one] Keep { $count } photograph
   *[other] Keep { $count } photographs
}
clean-bulk-reject-title = { $count ->
    [one] Reject { $count } photograph?
   *[other] Reject { $count } photographs?
}
clean-bulk-reject-body = { $count ->
    [one] It is left out of sampling and the video. The file is not moved or deleted.
   *[other] They are left out of sampling and the video. The files are not moved or deleted.
}
clean-bulk-reject-action = { $count ->
    [one] Reject { $count } photograph
   *[other] Reject { $count } photographs
}
clean-bulk-clear-title = { $count ->
    [one] Clear the decision on { $count } photograph?
   *[other] Clear the decisions on { $count } photographs?
}
clean-bulk-clear-body = { $count ->
    [one] It goes back to what the analysis proposes.
   *[other] They go back to what the analysis proposes.
}
clean-bulk-clear-action = { $count ->
    [one] Clear { $count } decision
   *[other] Clear { $count } decisions
}
clean-bulk-undo = You can change these decisions at any time.
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
files-failure-missing = the copy is no longer there
files-failure-modified = the copy was changed since it was made, so it was kept
files-failure-moved-since = the photograph was moved again since (for example into quarantine), so it was left where it is
files-failure-remove-failed = the copy could not be removed; it is still there
files-failure-now-an-original = an original was moved to this place since, so it was left there
files-failure-unrecorded-copy = the app closed while this file was being placed; what is at the target was left as it is
files-failure-replaced = the file there is no longer the copy this app made, so it was left there
files-failure-other = the operation failed

## Phase 4 — sampling
sampling-title = Sampling
sampling-intro = Choose which photographs become frames of the timelapse. Sampling follows the capture times, never the file order; photographs that are not selected are simply left out, never deleted.
sampling-no-project-description = Import a folder first; its photographs can then be sampled here.
sampling-method-title = Method
sampling-method-fixed-interval = Fixed interval
sampling-method-every-nth = Every Nth photo
sampling-method-uniform = Evenly spread
sampling-method-fixed-interval-help = One photograph for each step of the interval, starting from the first photograph. A step with no photograph close enough is reported, not filled.
sampling-method-every-nth-help = One photograph out of every N, in chronological order, starting from the offset (0 = the first photograph).
sampling-method-uniform-help = The chosen number of photographs, spread evenly over the period between the first and the last photograph.
sampling-interval = Interval
sampling-unit = Unit
sampling-unit-seconds = Seconds
sampling-unit-minutes = Minutes
sampling-unit-hours = Hours
sampling-unit-days = Days
sampling-policy = Which photograph
sampling-policy-closest = Closest to each step
sampling-policy-first-after = First one after each step
sampling-policy-last-before = Last one before each step
sampling-tolerance = Maximum gap from each step
sampling-tolerance-placeholder = Automatic
sampling-n = One photograph every
sampling-offset = Start at photograph number (0 = first)
sampling-count = Number of photographs
sampling-input-title = Photographs to choose from
sampling-from = From (date, UTC)
sampling-to = To (date, UTC, included)
sampling-all-dates = All dates
sampling-include-flagged = Include photographs still to review
sampling-input-note = Rejected and quarantined photographs are never used. Photographs without a capture date are left out and counted.
sampling-invalid = Check the values: numbers must be positive (whole numbers for counts) and the end date must not be before the start date.
sampling-preview = Preview
sampling-previewing = Calculating…
sampling-save = Save this selection
sampling-saving = Saving…
sampling-saved = { $count ->
    [one] Selection { $id } saved with one photograph.
   *[other] Selection { $id } saved with { $count } photographs.
}
sampling-preview-title = Preview
sampling-preview-empty-title = No preview yet
sampling-preview-empty-description = Choose a method and select Preview to see which photographs would be used.
sampling-stat-input = Photographs available
sampling-stat-selected = Photographs selected
sampling-stat-reduction = Reduction
sampling-stat-interval = Average interval
sampling-stat-unmatched = Steps without a photograph
sampling-stat-none = Not applicable
sampling-fps = Frames per second
sampling-fps-option = { $fps } fps
sampling-multiplier = Playback speed
sampling-multiplier-option = { $multiplier }×
sampling-duration = Estimated video length: { $duration }
sampling-excluded = Left out: { $rejected } rejected, { $quarantined } in quarantine, { $flagged } to review, { $undated } without a date, { $outside } outside the period.
sampling-nothing-selected-title = No photograph selected
sampling-nothing-selected-description = Widen the period, include the photographs still to review, or choose a smaller interval.
sampling-sequence-preview = { $count ->
    [one] Selected sequence: one photograph (preview)
   *[other] Selected sequence: { $count } photographs (preview)
}
sampling-sequence-run = { $count ->
    [one] Selected sequence: one photograph (selection { $id })
   *[other] Selected sequence: { $count } photographs (selection { $id })
}
sampling-strip-shown = Showing { $shown } of { $total }.
sampling-strip-preview-limit = The preview shows the first 200 photographs; save the selection to browse all of them.
sampling-runs-title = Saved selections
sampling-runs-empty-title = No saved selection
sampling-runs-empty-description = Save a preview to keep its exact sequence; the video will be made from a saved selection.
sampling-run-title = Selection { $id }
sampling-run-fixed-closest = Every { $interval }, closest photograph
sampling-run-fixed-first-after = Every { $interval }, first photograph after each step
sampling-run-fixed-last-before = Every { $interval }, last photograph before each step
sampling-run-nth = One photograph in { $n }
sampling-run-uniform = { $count } photographs spread evenly
sampling-run-stats = { $selected } of { $input } photographs
sampling-run-show = Show
sampling-run-verify = Check
sampling-verifying = Checking…
sampling-verify-identical = Still identical
sampling-verify-differs = Would now differ
sampling-verify-inputs-changed = The photographs have changed since

## Phase 5 — export
export-title = Export
export-intro = Turn a saved selection into an MP4 video. Encoding runs on this device with the FFmpeg bundled in the app; the video is saved in the “Timelapse exports” folder of the source folder and nothing is overwritten.
export-no-project-description = Import a folder first, then save a selection on the Sampling page.
export-no-ffmpeg-title = Video encoding is not available in this build
export-no-ffmpeg-description = The FFmpeg that ships with the app was not found. Install the app from an official package to export videos.
export-no-runs-title = No saved selection
export-no-runs-description = A video is made from a saved selection. Preview and save one on the Sampling page.
export-go-sampling = Go to Sampling
export-settings-title = Video
export-run = Selection
export-run-option = { $count ->
    [one] Selection { $id } (one photograph)
   *[other] Selection { $id } ({ $count } photographs)
}
export-codec = Format
export-codec-av1 = AV1 (recommended)
export-codec-h264 = H.264 (most compatible)
export-codec-hevc = HEVC / H.265
export-codec-unavailable = { $codec } (not available on this Mac)
export-codec-av1-help = AV1 gives the smallest files at a given quality and plays in recent browsers, VLC and macOS 14 or later.
export-codec-h264-help = H.264 plays almost everywhere. It is encoded by your Mac's video hardware; there is no slower software fallback.
export-codec-hevc-help = HEVC is smaller than H.264 and plays on Apple devices and most recent players. It is encoded by your Mac's video hardware.
export-resolution = Size
export-resolution-option = { $width } × { $height }
export-quality = Quality
export-quality-high = High (larger file)
export-quality-standard = Standard
export-quality-small = Smaller file
export-stat-photos = Photographs
export-stat-frames = Video frames
export-stat-duration = Video length
export-start = Create the video
export-exporting = Encoding…
export-cancel = Cancel
export-cancelling = Cancelling…
export-done-title = Video created
export-done = { $codec }, { $duration }, { $size }. It is saved at:
export-technical-details = Technical details
export-history-title = Videos made in this project
export-history-empty-title = No video yet
export-history-empty-description = Videos you create appear here with their settings and result.
export-status-running = Encoding
export-status-completed = Done
export-status-failed = Failed
export-status-cancelled = Cancelled
export-history-line = Selection { $run }, { $codec }, { $duration }
about-ffmpeg-notice = Video encoding uses FFmpeg (LGPL 2.1 or later) and SVT-AV1 (BSD-3-Clause-Clear licence and Alliance for Open Media Patent License 1.0). Their licence texts, build settings and source locations are included in the app, in Resources/resources/ffmpeg/LICENSES.

## Errors (rendered via i18n/errors.ts → t(`error-${kind}`, args)).
error-import-root-unreadable = The folder “{ $path }” could not be read.
error-image-unreadable = The photograph at “{ $path }” could not be read.
error-database = The project database could not be opened or written at “{ $path }”.
error-store-transaction = Saving to the project database failed.
error-image-undecodable = The photograph at “{ $path }” is not a readable image.
error-unknown-image = This photograph is no longer in the project.
error-invalid-sampling = { $reason ->
    [too-many-targets] This interval is too small for the period: it would create more than five million steps.
   *[other] The sampling settings must be positive numbers.
}
error-unknown-run = This saved selection no longer exists.
error-export = { $reason ->
    [ffmpeg-missing] The FFmpeg bundled with the app was not found.
    [encoder-unavailable] This format cannot be encoded on this Mac; choose another one. Nothing is substituted silently.
    [invalid-settings] These video settings are not supported (frame rate 1 to 120, speed 0.1 to 100, even sizes).
    [empty-run] This selection contains no photograph.
    [busy] Another video is already being created.
    [cancelled] The video was cancelled; the partial file was removed.
    [ffmpeg-failed] FFmpeg stopped with an error; the partial file was removed.
    [invalid-output] The finished file did not pass the check (format, size or frame count) and was discarded.
    [output-exists] A file with this name already exists; nothing was overwritten.
    [image-unreadable] A photograph of the selection could not be read.
    [write-failed] The video could not be written to the project's exports folder (disk full or not writable).
    [interrupted] The app closed while this video was being created.
    [unknown-export] This video no longer exists.
    [adjustment-stale] A photograph of this selection has changed since the chosen flicker or stabilization analysis. Analyse it again on the Adjust page.
    [adjustment-mismatch] A chosen flicker or stabilization analysis belongs to another selection, or does not cover every photograph of this one.
    [record-write-failed] The record could not be saved in the project’s exports folder (disk full or not writable).
   *[other] The video could not be created.
}
error-consolidation = { $reason ->
    [destination-invalid] Choose an existing folder, or a new one, by its full path (not a file).
    [destination-overlaps-collection] The working folder must be outside the imported collection, and must not contain it: the import would pick the copies up again.
    [destination-unwritable] The working folder could not be created or written to.
    [insufficient-space] There is not enough free space in the working folder for these photographs.
    [plan-changed] The folder or the photographs changed since the dry run. Run the dry run again, then confirm.
    [nothing-to-do] There is nothing to place: every photograph is already in the working folder, or skipped.
    [busy] Another organization or undo is already running.
    [unknown-consolidation] This organization no longer exists.
    [already-undone] This organization has already been undone.
   *[other] The photographs could not be organized.
}

organize-title = Organize
organize-no-project-description = Import a folder of photographs first; you can then gather them into one working folder.
organize-intro = Gather photographs from the collection's folders into one working folder, as copies or by moving the originals. A dry run shows exactly what will happen before anything is written, every file is checked, nothing is ever overwritten, and each run can be undone.
organize-settings-title = Settings
organize-destination = Working folder
organize-destination-none = No folder chosen
organize-destination-choose = Choose a folder…
organize-destination-help = Choose a folder outside the imported collection, for example on another drive.
organize-operation = Operation
organize-operation-copy = Copy (the originals stay)
organize-operation-move = Move the originals
organize-operation-copy-help = Each photograph is copied; on a Mac drive the copy takes no extra space until it is edited. Editing a copy never changes the original.
organize-operation-move-help = Each original moves to the working folder once its copy there is verified; the project follows it. You can move it back with Undo.
organize-source = Photographs
organize-source-all = All, except rejected and quarantined
organize-naming = Names
organize-naming-preserve = Keep the folder structure
organize-naming-original = Original names, in one folder
organize-naming-folder-prefix = Folder name + original name
organize-naming-sequence = Sequence numbers
organize-naming-preserve-help = The collection's subfolders are recreated in the working folder.
organize-naming-original-help = All photographs go into one folder under their own names; a name already taken gets _2, _3…
organize-naming-folder-prefix-help = All photographs go into one folder, named after their source folder (for example 157GOPRO_G0010001.JPG).
organize-naming-sequence-help = All photographs go into one folder as 000001.jpg, 000002.jpg… in chronological order, or in the order of the selection.
organize-preview = Dry run
organize-previewing = Planning…
organize-preview-title = Dry run
organize-stat-planned = To place
organize-stat-duplicates = Duplicates skipped
organize-stat-present = Already there
organize-stat-renamed = Renamed
organize-stat-bytes = Size
organize-stat-free = Free space
organize-free-unknown = Unknown
organize-excluded = Left out: { $rejected } rejected, { $quarantined } quarantined, { $missing } not found where expected.
organize-unsafe-names = { $count ->
    [one] One photograph has a name that cannot be used safely; it is skipped.
   *[other] { $count } photographs have names that cannot be used safely; they are skipped.
}
organize-items-title = First { $shown } of { $total } photographs
organize-notable-title = Skipped or renamed ({ $count })
organize-item-renamed = Renamed: the name was already taken
organize-item-duplicate = Skipped: identical to an earlier photograph
organize-item-already-present = Skipped: an identical file is already there
organize-item-unsafe = Skipped: its name cannot be used safely
organize-run-copy = { $count ->
    [one] Copy one photo
   *[other] Copy { $count } photos
}
organize-run-move = { $count ->
    [one] Move one photo
   *[other] Move { $count } photos
}
organize-preview-stale = The settings changed since the dry run. Run it again to see what will happen.
organize-busy-run = Organizing…
organize-busy-undo = Undoing…
organize-confirm-title-copy = Copy the photographs?
organize-confirm-title-move = Move the originals?
organize-confirm-body-copy = { $count ->
    [one] One photograph will be copied to { $path }.
   *[other] { $count } photographs will be copied to { $path }.
}
organize-confirm-body-move = { $count ->
    [one] One original will be moved to { $path }. It will no longer be in the collection's folder.
   *[other] { $count } originals will be moved to { $path }. They will no longer be in the collection's folders.
}
organize-confirm-safety = Each file is checked before and after; nothing is ever overwritten, and the run can be undone from the history.
organize-confirm-move-understood = I understand the originals will leave the collection's folders
organize-result-copy = { $failed ->
    [0] { $placed } photographs copied.
   *[other] { $placed } photographs copied; { $failed } could not be (see the history).
}
organize-result-move = { $failed ->
    [0] { $placed } originals moved.
   *[other] { $placed } originals moved; { $failed } could not be (see the history).
}
organize-undo-result = { $failed ->
    [0] Undone: { $undone } files taken back.
   *[other] { $undone } files taken back; { $failed } were left as they are (changed or moved since).
}
organize-history-title = History
organize-history-empty-title = Nothing organized yet
organize-history-empty-description = Each copy or move appears here, with a check and an undo.
organize-history-line-copy = { $count ->
    [one] One photograph copied, { $naming }
   *[other] { $count } photographs copied, { $naming }
}
organize-history-line-move = { $count ->
    [one] One original moved, { $naming }
   *[other] { $count } originals moved, { $naming }
}
organize-history-failed = { $count } failed
organize-history-undone = { $count } undone
organize-status-running = Running
organize-status-completed = Done
organize-status-partial = Partly done
organize-status-cancelled = Cancelled
organize-status-interrupted = Interrupted
organize-undo-status-undone = Undone
organize-undo-status-partially-undone = Partly undone
organize-check = Check
organize-checking = Checking…
organize-check-ok = { $count ->
    [one] The file is intact.
   *[other] All { $count } files are intact.
}
organize-check-problems = { $ok } intact, { $modified } changed, { $missing } missing, { $moved } moved since.
organize-undo = Undo
organize-undo-title = Undo this organization?
organize-undo-body-copy = { $count ->
    [one] The copy made in { $path } will be removed.
   *[other] The { $count } copies made in { $path } will be removed.
}
organize-undo-body-move = { $count ->
    [one] The original will be moved back from { $path } to where it was.
   *[other] The { $count } originals will be moved back from { $path } to where they were.
}
organize-undo-safety = Only files still exactly as they were made are taken back; anything changed since is left in place and listed. Nothing else in the folder is touched.
organize-originals-title = Originals check
organize-originals-help = Re-reads every original where the project says it is and compares it with its fingerprint from the import.
organize-originals-check = Check the originals
organize-originals-ok = Intact
organize-originals-changed = Changed
organize-originals-missing = Missing
organize-originals-unhashed = Not checkable
organize-originals-all-ok = { $count ->
    [one] The original is intact.
   *[other] All { $count } originals are intact.
}
organize-originals-problems = { $changed } changed and { $missing } missing. The app never writes to an original: these were changed or removed outside the app.
organize-problem-modified = Changed since it was made
organize-problem-missing = Not found
organize-problem-moved-since = Moved again since
organize-problem-changed = Changed since the import

## Phase 7 — contact sheets, timeline periods, export record

sheets-title = Contact sheets
sheets-no-project-description = Import a folder of photographs first; you can then print them as contact sheets.
sheets-intro = A contact sheet is a PDF of thumbnails with their file names and capture times, made from the project’s records. It goes into the “Timelapse exports/contact-sheets” folder of the source folder.
sheets-source-title = What to show
sheets-source = Photographs
sheets-source-all = All photographs
sheets-source-period = A period
sheets-source-flagged = Flagged photographs
sheets-source-quarantined = Quarantined photographs
sheets-source-sampling = A saved selection
sheets-source-export = The sequence of a video
sheets-source-period-line = From { $from } to { $to } (UTC)
sheets-source-sampling-line = Selection #{ $id }
sheets-source-export-line = Video #{ $id }
sheets-period-from = From (UTC)
sheets-period-to = Until, not included (UTC)
sheets-period-help = Photographs taken from the first time up to, but not including, the second.
sheets-period-invalid = Choose a start and an end; the end must come after the start.
sheets-sampling-run = Selection
sheets-no-runs = No selection saved yet. Save one on the Sampling page.
sheets-export = Video
sheets-export-option = Video #{ $id } (selection #{ $run }, { $count } photographs)
sheets-no-exports = No video created yet.
sheets-layout-title = Layout
sheets-page-size = Paper
sheets-page-a4 = A4
sheets-page-letter = Letter
sheets-orientation = Orientation
sheets-orientation-portrait = Portrait
sheets-orientation-landscape = Landscape
sheets-columns = Columns
sheets-rows = Rows
sheets-show-filename = File name
sheets-show-timestamp = Capture time
sheets-times-utc = Capture times are printed in UTC, as the app shows them.
sheets-preview-title = First page
sheets-preview-none-title = Nothing to preview yet
sheets-preview-none-description = Finish choosing the photographs to see the first page.
sheets-preview-label = { $count ->
    [one] First page of the sheet, { $count } photograph
   *[other] First page of the sheet, { $count } photographs
}
sheets-preview-help = Hover a thumbnail to see its caption. The PDF uses the same grid.
sheets-stat-photos = Photographs
sheets-stat-pages = Pages
sheets-stat-per-page = Per page
sheets-empty-title = No photograph here
sheets-empty-description = These photographs are not in the project, or none matches.
sheets-generate = Create the contact sheet
sheets-generating = Creating…
sheets-done-title = Contact sheet created
sheets-done = { $pages ->
    [one] { $pages } page
   *[other] { $pages } pages
}, { $count ->
    [one] { $photos } photograph
   *[other] { $photos } photographs
}, { $size }.
sheets-unreadable = { $count ->
    [one] { $count } photograph could not be read; its place is marked on the sheet.
   *[other] { $count } photographs could not be read; their places are marked on the sheet.
}
sheets-other-running = Another project’s contact sheet is being created; you can create this one when it ends.
sheets-cancelled = The contact sheet was cancelled; the partial file was removed.
sheets-history-title = Contact sheets created
sheets-history-empty-title = No contact sheet yet
sheets-history-empty-description = The sheets you create are listed here, with their file.
sheets-history-line = { $pages ->
    [one] { $pages } page
   *[other] { $pages } pages
}, { $count ->
    [one] { $photos } photograph
   *[other] { $photos } photographs
}
sheets-status-running = Running
sheets-status-completed = Created
sheets-status-failed = Failed
sheets-status-cancelled = Cancelled
sheets-status-interrupted = Interrupted
# Printed on the PDF.
sheets-pdf-title = Contact sheet
sheets-pdf-source = { $count ->
    [one] { $source } · { $photos } photograph
   *[other] { $source } · { $photos } photographs
}
sheets-pdf-page = Page { $page } of { $pages }
sheets-pdf-note = Capture times in UTC · Timelapse Studio
sheets-pdf-unreadable = Not readable

timeline-zoom-all = Whole collection
timeline-zoom-help = Choose a bar to see its photographs and zoom into it.
timeline-pick-help = Choose a bar to see its photographs.
timeline-period-title = Photographs of { $period }
timeline-period-count = { $count ->
    [one] { $photos } photograph in this period
   *[other] { $photos } photographs in this period
}
timeline-period-sheet = Contact sheet of this period
timeline-identical-none = No two photographs share the same capture time.
timeline-identical-title = { $groups ->
    [one] { $groups } capture time shared by several photographs ({ $photos } photographs)
   *[other] { $groups } capture times shared by several photographs ({ $photos } photographs)
}
timeline-identical-help = Often a burst, sometimes a camera whose clock was not set, or copies. Nothing is changed.
timeline-identical-count = { $count ->
    [one] { $photos } photograph
   *[other] { $photos } photographs
}
timeline-identical-more = { $count ->
    [one] And { $count } more time.
   *[other] And { $count } more times.
}

clean-sheet-flagged = Contact sheet of flagged
clean-sheet-quarantined = Contact sheet of quarantined
sampling-run-sheet = Contact sheet
export-sheet = Contact sheet
export-record-show = Show record
export-record-hide = Hide record
export-record-save = Save record
export-record-saved = Record saved:
export-record-help = Made from the project’s records each time it is shown; the app never reads it back.
export-record-label = Record of video #{ $id }

error-timeout = The app took too long to answer. The work may still be running: wait a moment, then try again.
error-contact-sheet = { $reason ->
    [busy] Another contact sheet is already being created.
    [empty] There is no photograph to show for this choice.
    [too-many] A contact sheet can show at most 100,000 photographs; choose fewer.
    [invalid-period] Choose a start and an end; the end must come after the start.
    [unknown-export] This video no longer exists.
    [source-changed] The photographs changed since the preview (for example a status changed). Check the preview, then create the sheet again.
    [write-failed] The contact sheet could not be written to the project’s folder (disk full or not writable).
    [output-exists] A file with this name already exists; nothing was overwritten.
    [interrupted] The app closed while this contact sheet was being created.
   *[other] The contact sheet could not be created.
}

## Phase 8 — deflicker and stabilization

adjust-title = Adjust
adjust-no-project-description = Import a folder of photographs and save a selection first; you can then reduce flicker and shake.
adjust-no-runs-description = Flicker reduction and stabilization work on a saved selection. Save one on the Sampling page.
adjust-intro = Measure a saved selection, check the result here, then choose it when you export. Your photographs are never changed: the corrections are applied while the video is made.
adjust-flicker-title = Flicker reduction
adjust-flicker-help = Follows the slow changes of light (sunrise, sunset, clouds) and evens out the quick jumps in exposure between photographs. Corrections are limited to one stop of exposure and a third of a stop of colour.
adjust-strength = Strength
adjust-window = Smoothing
adjust-frames-option = { $count ->
    [one] { $count } frame
   *[other] { $count } frames
}
adjust-colour = Also even out colour (white balance)
adjust-analyse = Analyse
adjust-analysing = Analysing…
adjust-stopped = The analysis was stopped; nothing was stored.
adjust-other-running = Another project is being analysed; you can start here when it ends.
adjust-result = Result shown
adjust-flicker-option = #{ $id } · { $date } · { $strength } · { $window } frames
adjust-stab-option = #{ $id } · { $date } · crop { $crop }
adjust-stat-largest = Largest correction
adjust-stat-mean = Average correction
adjust-stat-colour = Largest colour correction
adjust-stat-measured = Measured
adjust-measured-of = { $measured } of { $frames }
adjust-stops = { $value } EV
adjust-none-title = Nothing analysed yet
adjust-flicker-none = Analyse this selection to see its brightness, before and after.
adjust-stab-none = Analyse this selection to see the camera's movement, before and after.
adjust-chart-brightness = Brightness from frame to frame (stops from typical)
adjust-chart-red = Red balance (stops from typical)
adjust-chart-blue = Blue balance (stops from typical)
adjust-chart-horizontal = Horizontal position (share of the frame)
adjust-chart-vertical = Vertical position (share of the frame)
adjust-series-before = As shot
adjust-series-trend = Trend kept
adjust-series-after = Corrected
adjust-series-path = Camera as shot
adjust-series-stabilized = Stabilized
adjust-axis-frame = Frame
adjust-show-table = Show the values as a table
# A chart value in its tooltip and table: series name, then the value.
chart-value = { $label }: { $value }
chart-no-value = —
adjust-stab-title = Stabilization
adjust-stab-help = Measures how the picture moves from one photograph to the next and keeps the camera on a smooth path. Shifts only (no rotation); every frame is cropped by the same small amount so the edges never show.
adjust-smoothing = Smoothing
adjust-max-crop = Largest crop per side
adjust-stat-crop = Crop per side
adjust-stat-shift = Largest shift
adjust-stat-pairs = Movements measured
adjust-stab-few = Few movements could be measured with confidence (featureless or very different photographs): stabilization will do little on this selection.
adjust-compare-title = Before and after
adjust-compare-none = Analyse flicker or stabilization to compare a photograph with its corrected version.
adjust-frame = { $count ->
    [one] Frame (of { $count })
   *[other] Frame (of { $count })
}
adjust-previous = Previous
adjust-next = Next
adjust-apply-flicker = Flicker reduction
adjust-apply-stab = Stabilization
adjust-label-before = As shot
adjust-label-after = Corrected
adjust-as-shot = Frame { $frame } as shot
adjust-adjusted = Frame { $frame } corrected
export-flicker = Flicker reduction
export-stab = Stabilization
export-adjust-none = None
export-flicker-option = #{ $id } · strength { $strength }
export-stab-option = #{ $id } · crop { $crop }

error-adjust = { $reason ->
    [busy] Another analysis is already running.
    [empty-run] This selection contains no photograph.
    [unknown-adjustment] This analysis no longer exists.
    [adjustment-mismatch] This analysis belongs to another selection, or does not cover every photograph of this one. Analyse this selection again.
    [adjustment-stale] A photograph of this selection has changed since it was analysed. Analyse the selection again.
    [unknown-frame] This frame is not part of the selection.
   *[other] The analysis could not be completed.
}
