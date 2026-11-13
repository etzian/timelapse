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

## Errors (rendered via i18n/errors.ts → t(`error-${kind}`, args)).
error-import-root-unreadable = The folder “{ $path }” could not be read.
error-image-unreadable = A photograph at “{ $path }” could not be hashed.
error-database = The project database could not be opened or written at “{ $path }”.
error-store-transaction = Saving the import into the project database failed.
