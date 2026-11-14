## Chaînes propres à l’application. Les chaînes partagées (actions, erreurs,
## réglages, juridique, consentement) viennent de _foundation/i18n/common et
## se trouvent à côté de ce fichier.

-app-name = Timelapse Studio
app-name = { -app-name }
app-tagline = Transformez d’importants fichiers de photos en vidéos timelapse de haute qualité, localement.
company-name = Etzian inc.

nav-home = Accueil
nav-settings = Réglages

home-title = Importer une collection
home-intro = Choisissez le dossier qui contient vos photographies. Timelapse Studio l’explore, applique un hachage SHA-256 aux formats pris en charge (JPG, PNG, TIFF), cherche les doublons de contenu et consigne tout cela dans une base de données projet sur cet appareil.
home-import-action = Choisir un dossier à importer
home-importing = Importation…
home-stat-scanned = Fichiers explorés
home-stat-hashed = Fichiers hachés
home-stat-new = Nouvelles entrées
home-stat-unchanged = Déjà présentes
home-stat-duplicates = Fichiers en doublon
home-skipped-unreadable = { $count } fichier(s) n’ont pas pu être lus et ont été importés sans hachage.
home-duplicate-groups-summary = Groupes de doublons trouvés lors de cet import
home-integrity-missing = { $count } entrée(s) de l’inventaire n’a/ont plus de fichier sur disque
home-integrity-dups = { $count } groupe(s) distinct(s) de doublon(s) dans l’inventaire
home-root = Dossier source : { $path }

settings-section-privacy = Confidentialité
settings-privacy-description = Tout reste sur cet appareil. Rien n’est transmis.
erase-item-settings = Vos choix de langue, de thème et de densité

about-endorsed-by = par

## Phase 2 — chronologie
timeline-dating = Lecture des dates de prise de vue…
timeline-backfill-result = { $resolved ->
    [one] { $resolved } image a été datée lors de ce passage
   *[other] { $resolved } images ont été datées lors de ce passage
}{ $unresolved ->
    [0] .
    [one] ; { $unresolved } n’a pas pu être datée.
   *[other] ; { $unresolved } n’ont pas pu être datées.
}
timeline-title = Chronologie
timeline-summary-resolved = Images datées
timeline-summary-unresolved = Images sans date
timeline-summary-low-confidence = Fiabilité faible
timeline-summary-tz-assumed = Fuseau horaire non consigné
timeline-by-source = Provenance des dates
timeline-source-exif-original = Horloge de l’appareil (EXIF, original)
timeline-source-exif-digitized = Horloge de l’appareil (EXIF, numérisation)
timeline-source-filename = Nom du fichier
timeline-source-filesystem = Date de modification du fichier
timeline-confidence-high = Fiabilité élevée
timeline-confidence-medium = Fiabilité moyenne
timeline-confidence-low = Fiabilité faible
timeline-tz-assumed = Fuseau non consigné
timeline-unresolved = Sans date
timeline-dates-utc = Les dates sont affichées en UTC. Quand un appareil n’a pas consigné son fuseau horaire, son horloge est lue comme UTC; aucune date n’est jamais corrigée.
timeline-empty-title = Aucune image datée pour l’instant
timeline-empty-description = Dès qu’une photographie importée a une date de prise de vue (appareil, nom ou date du fichier), elle apparaît ici en ordre chronologique.
timeline-distribution = Répartition sur la période
timeline-granularity-hour = Par heure
timeline-granularity-day = Par jour
timeline-granularity-month = Par mois
timeline-gaps-title = { $count ->
    [one] { $count } intervalle anormalement long
   *[other] { $count } intervalles anormalement longs
}
timeline-gaps-none = Aucun intervalle anormalement long (seuil : { $threshold }).
timeline-gap = { $from } → { $to } ({ $duration })
timeline-gap-source-changed = Les dates de part et d’autre proviennent de sources différentes : il peut s’agir d’une incohérence des métadonnées plutôt que d’une pause dans la prise de vue.
timeline-list-title = Images en ordre chronologique
timeline-list-truncated = { $shown } images affichées sur { $total }.
timeline-show-more = Afficher plus
timeline-loading = Construction de la chronologie…
timeline-retry = Réessayer

## Erreurs (rendues via i18n/errors.ts → t(`error-${kind}`, args)).
error-import-root-unreadable = Le dossier « { $path } » n’a pas pu être lu.
error-image-unreadable = Une photographie située à « { $path } » n’a pas pu être hachée.
error-database = La base de données du projet n’a pas pu être ouverte ou écrite à l’emplacement « { $path } ».
error-store-transaction = L’enregistrement de l’import dans la base de données du projet a échoué.
