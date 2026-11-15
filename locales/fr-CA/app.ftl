## Chaînes propres à l’application. Les chaînes partagées (actions, erreurs,
## réglages, juridique, consentement) viennent de _foundation/i18n/common et
## se trouvent à côté de ce fichier.

-app-name = Timelapse Studio
app-name = { -app-name }
app-tagline = Transformez d’importants fichiers de photos en vidéos timelapse de haute qualité, localement.
company-name = Etzian inc.

nav-home = Accueil
nav-cleaning = Nettoyage
nav-sampling = Échantillonnage
nav-export = Exportation
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

## Phase 3 — nettoyage
clean-title = Nettoyage
clean-intro = Timelapse Studio mesure la luminosité de chaque photographie et signale celles qui semblent trop sombres ou que Lightroom a marquées comme rejetées. Rien n’est retiré automatiquement : vous décidez, et une photo rejetée ne va dans la quarantaine du projet que lorsque vous le confirmez.
clean-no-project-title = Aucune collection ouverte
clean-no-project-description = Importez d’abord un dossier; ses photographies pourront ensuite être analysées et nettoyées ici.
clean-go-import = Aller à l’importation
clean-thresholds-title = Seuils
clean-threshold-min-mean = Luminosité moyenne minimale (0–255)
clean-threshold-dark-level = Niveau d’un pixel sombre (0–255)
clean-threshold-max-dark = Proportion maximale de pixels sombres (%)
clean-threshold-xmp = Signaler les photos rejetées dans Lightroom
clean-thresholds-invalid = Entrez des nombres entiers ou décimaux dans les plages indiquées (le niveau sombre doit être un nombre entier).
clean-thresholds-note = Modifier un seuil réévalue les photos sur lesquelles vous ne vous êtes pas prononcé; vos propres décisions sont conservées.
clean-apply = Appliquer les seuils
clean-applying = Application…
clean-analysis-title = Analyse
clean-analyze = Analyser les photographies
clean-analyzing = Analyse en cours…
clean-progress = { $done } sur { $total }
clean-loading = Chargement…
clean-no-preview = Aucun aperçu
clean-retry = Réessayer
clean-dismiss = Fermer
clean-status-eligible = Conservée
clean-status-flagged = À examiner
clean-status-rejected = Rejetée
clean-status-quarantined = En quarantaine
clean-status-unanalyzed = Non analysée
clean-filter-label = Afficher
clean-filter-flagged = À examiner
clean-filter-rejected = Rejetées
clean-filter-quarantined = Quarantaine
clean-filter-all = Toutes
clean-filter-empty-flagged = Rien à examiner
clean-filter-empty-rejected = Aucune photographie rejetée
clean-filter-empty-quarantined = La quarantaine est vide
clean-filter-empty-all = Aucune photographie
clean-filter-empty-description = Les autres photographies se trouvent sous les autres onglets.
clean-show-all = Tout afficher
clean-empty-title = Aucune photographie dans cette collection
clean-empty-description = Importez un dossier qui contient des fichiers JPG, PNG ou TIFF.
clean-unanalyzed-title = Pas encore analysée
clean-unanalyzed-description = Analysez la collection pour mesurer la luminosité de chaque photographie.
clean-inspect = Examiner { $path }
clean-user-decision = Votre décision
clean-metrics = Moyenne { $mean } · { $dark } sombre
clean-accept = Conserver
clean-reject = Rejeter
clean-clear = Annuler la décision
clean-restore = Restaurer
clean-shown = { $shown } affichées sur { $total }.
clean-show-more = Afficher plus
clean-quarantine-action = { $count ->
    [one] Mettre { $count } photo rejetée en quarantaine…
   *[other] Mettre { $count } photos rejetées en quarantaine…
}
clean-confirm-title = Mettre en quarantaine?
clean-confirm-body = { $count ->
    [one] { $count } photographie sera déplacée vers { $path }.
   *[other] { $count } photographies seront déplacées vers { $path }.
}
clean-confirm-safety = Chaque fichier est vérifié avant et après le déplacement et peut être restauré en tout temps depuis l’onglet Quarantaine. Rien n’est supprimé.
clean-confirm-action = { $count ->
    [one] Déplacer { $count } photo
   *[other] Déplacer { $count } photos
}
clean-cancel = Annuler
clean-moving = Déplacement…
clean-quarantined-result = { $ok ->
    [one] { $ok } photographie mise en quarantaine
   *[other] { $ok } photographies mises en quarantaine
}{ $failed ->
    [0] .
    [one] ; { $failed } n’a pas pu être déplacée :
   *[other] ; { $failed } n’ont pas pu être déplacées :
}
clean-restored-result = { $ok ->
    [one] { $ok } photographie restaurée
   *[other] { $ok } photographies restaurées
}{ $failed ->
    [0] .
    [one] ; { $failed } n’a pas pu être restaurée :
   *[other] ; { $failed } n’ont pas pu être restaurées :
}
clean-detail-status = Statut
clean-detail-mean = Luminosité moyenne
clean-detail-median = Luminosité médiane
clean-detail-dark = Pixels sombres
clean-detail-xmp = Cote Lightroom
clean-detail-location = En quarantaine à
clean-detail-histogram = Histogramme de luminosité
clean-reason-mean-below = Luminosité moyenne de { $mean }, sous le seuil de { $min }
clean-reason-too-many-dark-pixels = { $fraction } des pixels sont sombres (limite : { $max })
clean-reason-xmp-rejected = Rejetée dans Lightroom
clean-reason-analysis-failed = Analyse impossible : { $failure }
clean-failure-unreadable = le fichier n’a pas pu être lu
clean-failure-undecodable = le fichier n’est pas une image lisible
files-failure-unknown-image = cette photographie ne fait plus partie du projet
files-failure-not-rejected = seules les photographies rejetées peuvent être mises en quarantaine
files-failure-not-quarantined = cette photographie n’est pas en quarantaine
files-failure-unsafe-path = son chemin n’a pas pu être placé de façon sûre dans la quarantaine
files-failure-source-unreadable = le fichier n’a pas pu être lu
files-failure-changed-since-import = le fichier a changé depuis son importation; il n’a donc pas été déplacé
files-failure-destination-exists = un fichier existe déjà à la destination; rien n’a été écrasé
files-failure-move-failed = le fichier n’a pas pu être déplacé; il est toujours au même endroit
files-failure-source-in-use = le fichier est en cours d’utilisation ou en lecture seule; il est toujours au même endroit
files-failure-record-failed = le fichier a été déplacé, mais pas encore consigné; l’application règle cela automatiquement
files-failure-verify-failed = la copie ne correspondait pas à l’original et a été écartée; l’original est toujours au même endroit
files-failure-interrupted = le déplacement a été interrompu et annulé; le fichier est toujours au même endroit
files-failure-inconsistent = le déplacement a été interrompu dans un état inattendu; aucun fichier n’a été touché
files-failure-other = l’opération a échoué

## Phase 4 — échantillonnage
sampling-title = Échantillonnage
sampling-intro = Choisissez les photographies qui deviendront les images du timelapse. L’échantillonnage suit les dates de prise de vue, jamais l’ordre des fichiers; les photographies non retenues sont simplement laissées de côté, jamais supprimées.
sampling-no-project-description = Importez d’abord un dossier; ses photographies pourront ensuite être échantillonnées ici.
sampling-method-title = Méthode
sampling-method-fixed-interval = Intervalle fixe
sampling-method-every-nth = Une photo sur N
sampling-method-uniform = Répartition uniforme
sampling-method-fixed-interval-help = Une photographie par pas de l’intervalle, à partir de la première photographie. Un pas sans photographie assez proche est signalé, jamais comblé.
sampling-method-every-nth-help = Une photographie sur N, en ordre chronologique, à partir du décalage (0 = la première photographie).
sampling-method-uniform-help = Le nombre choisi de photographies, réparties uniformément entre la première et la dernière photographie.
sampling-interval = Intervalle
sampling-unit = Unité
sampling-unit-seconds = Secondes
sampling-unit-minutes = Minutes
sampling-unit-hours = Heures
sampling-unit-days = Jours
sampling-policy = Quelle photographie
sampling-policy-closest = La plus proche de chaque pas
sampling-policy-first-after = La première après chaque pas
sampling-policy-last-before = La dernière avant chaque pas
sampling-tolerance = Écart maximal par rapport à chaque pas
sampling-tolerance-placeholder = Automatique
sampling-n = Une photographie toutes les
sampling-offset = Commencer à la photographie numéro (0 = la première)
sampling-count = Nombre de photographies
sampling-input-title = Photographies parmi lesquelles choisir
sampling-from = Du (date, UTC)
sampling-to = Au (date, UTC, incluse)
sampling-include-flagged = Inclure les photographies à examiner
sampling-input-note = Les photographies rejetées ou en quarantaine ne sont jamais utilisées. Les photographies sans date de prise de vue sont laissées de côté et comptées.
sampling-invalid = Vérifiez les valeurs : les nombres doivent être positifs (entiers pour les nombres de photos) et la date de fin ne peut pas précéder la date de début.
sampling-preview = Aperçu
sampling-previewing = Calcul en cours…
sampling-save = Enregistrer cette sélection
sampling-saving = Enregistrement…
sampling-saved = { $count ->
    [one] Sélection { $id } enregistrée avec { $count } photographie.
   *[other] Sélection { $id } enregistrée avec { $count } photographies.
}
sampling-preview-title = Aperçu
sampling-preview-empty-title = Aucun aperçu pour l’instant
sampling-preview-empty-description = Choisissez une méthode, puis Aperçu pour voir quelles photographies seraient utilisées.
sampling-stat-input = Photographies disponibles
sampling-stat-selected = Photographies retenues
sampling-stat-reduction = Réduction
sampling-stat-interval = Intervalle moyen
sampling-stat-unmatched = Pas sans photographie
sampling-stat-none = Sans objet
sampling-fps = Images par seconde
sampling-fps-option = { $fps } i/s
sampling-multiplier = Vitesse de lecture
sampling-multiplier-option = { $multiplier } ×
sampling-duration = Durée estimée de la vidéo : { $duration }
sampling-excluded = Laissées de côté : { $rejected } rejetées, { $quarantined } en quarantaine, { $flagged } à examiner, { $undated } sans date, { $outside } hors de la période.
sampling-nothing-selected-title = Aucune photographie retenue
sampling-nothing-selected-description = Élargissez la période, incluez les photographies à examiner ou choisissez un intervalle plus court.
sampling-sequence-preview = { $count ->
    [one] Séquence retenue : { $count } photographie (aperçu)
   *[other] Séquence retenue : { $count } photographies (aperçu)
}
sampling-sequence-run = { $count ->
    [one] Séquence retenue : { $count } photographie (sélection { $id })
   *[other] Séquence retenue : { $count } photographies (sélection { $id })
}
sampling-strip-shown = { $shown } affichées sur { $total }.
sampling-strip-preview-limit = L’aperçu montre les 200 premières photographies; enregistrez la sélection pour les parcourir toutes.
sampling-runs-title = Sélections enregistrées
sampling-runs-empty-title = Aucune sélection enregistrée
sampling-runs-empty-description = Enregistrez un aperçu pour conserver sa séquence exacte; la vidéo sera produite à partir d’une sélection enregistrée.
sampling-run-title = Sélection { $id }
sampling-run-fixed-closest = Intervalle de { $interval }, photographie la plus proche
sampling-run-fixed-first-after = Intervalle de { $interval }, première photographie après chaque pas
sampling-run-fixed-last-before = Intervalle de { $interval }, dernière photographie avant chaque pas
sampling-run-nth = Une photographie sur { $n }
sampling-run-uniform = { $count } photographies réparties uniformément
sampling-run-stats = { $selected } sur { $input } photographies
sampling-run-show = Afficher
sampling-run-verify = Vérifier
sampling-verifying = Vérification…
sampling-verify-identical = Toujours identique
sampling-verify-differs = Différerait maintenant
sampling-verify-inputs-changed = Les photographies ont changé depuis

## Phase 5 — exportation
export-title = Exportation
export-intro = Transformez une sélection enregistrée en vidéo MP4. L’encodage se fait sur cet appareil avec le FFmpeg fourni avec l’application; la vidéo est enregistrée dans le dossier .timelapse/exports du projet et rien n’est écrasé.
export-no-project-description = Importez d’abord un dossier, puis enregistrez une sélection dans la page Échantillonnage.
export-no-ffmpeg-title = L’encodage vidéo n’est pas offert dans cette version
export-no-ffmpeg-description = Le FFmpeg fourni avec l’application est introuvable. Installez l’application à partir d’un paquet officiel pour exporter des vidéos.
export-no-runs-title = Aucune sélection enregistrée
export-no-runs-description = Une vidéo est produite à partir d’une sélection enregistrée. Prévisualisez-en une et enregistrez-la dans la page Échantillonnage.
export-go-sampling = Aller à l’échantillonnage
export-settings-title = Vidéo
export-run = Sélection
export-run-option = { $count ->
    [one] Sélection { $id } ({ $count } photographie)
   *[other] Sélection { $id } ({ $count } photographies)
}
export-codec = Format
export-codec-av1 = AV1 (recommandé)
export-codec-h264 = H.264 (le plus compatible)
export-codec-hevc = HEVC / H.265
export-codec-unavailable = { $codec } (non offert sur ce Mac)
export-codec-av1-help = L’AV1 produit les fichiers les plus légers à qualité égale et se lit dans les navigateurs récents, VLC et macOS 14 ou ultérieur.
export-codec-h264-help = Le H.264 se lit presque partout. Il est encodé par le matériel vidéo de votre Mac; aucun encodage plus lent par le processeur ne prend le relais.
export-codec-hevc-help = Le HEVC est plus léger que le H.264 et se lit sur les appareils Apple et la plupart des lecteurs récents. Il est encodé par le matériel vidéo de votre Mac.
export-resolution = Taille
export-resolution-option = { $width } × { $height }
export-quality = Qualité
export-quality-high = Élevée (fichier plus lourd)
export-quality-standard = Standard
export-quality-small = Fichier plus léger
export-stat-photos = Photographies
export-stat-frames = Images de la vidéo
export-stat-duration = Durée de la vidéo
export-start = Créer la vidéo
export-exporting = Encodage…
export-cancel = Annuler
export-cancelling = Annulation…
export-done-title = Vidéo créée
export-done = { $codec }, { $duration }, { $size }. Elle est enregistrée ici :
export-technical-details = Détails techniques
export-history-title = Vidéos produites dans ce projet
export-history-empty-title = Aucune vidéo pour l’instant
export-history-empty-description = Les vidéos que vous créez apparaissent ici avec leurs réglages et leur résultat.
export-status-running = Encodage
export-status-completed = Terminée
export-status-failed = Échec
export-status-cancelled = Annulée
export-history-line = Sélection { $run }, { $codec }, { $duration }
about-ffmpeg-notice = L’encodage vidéo utilise FFmpeg (LGPL 2.1 ou ultérieure) et SVT-AV1 (licence BSD-3-Clause-Clear et Alliance for Open Media Patent License 1.0). Leurs textes de licence, réglages de compilation et emplacements des sources sont inclus dans l’application, dans Resources/resources/ffmpeg/LICENSES.

## Erreurs (rendues via i18n/errors.ts → t(`error-${kind}`, args)).
error-import-root-unreadable = Le dossier « { $path } » n’a pas pu être lu.
error-image-unreadable = La photographie située à « { $path } » n’a pas pu être lue.
error-database = La base de données du projet n’a pas pu être ouverte ou écrite à l’emplacement « { $path } ».
error-store-transaction = L’enregistrement dans la base de données du projet a échoué.
error-image-undecodable = La photographie située à « { $path } » n’est pas une image lisible.
error-unknown-image = Cette photographie ne fait plus partie du projet.
error-invalid-sampling = { $reason ->
    [too-many-targets] Cet intervalle est trop court pour la période : il créerait plus de cinq millions de pas.
   *[other] Les réglages d’échantillonnage doivent être des nombres positifs.
}
error-unknown-run = Cette sélection enregistrée n’existe plus.
error-export = { $reason ->
    [ffmpeg-missing] Le FFmpeg fourni avec l’application est introuvable.
    [encoder-unavailable] Ce format ne peut pas être encodé sur ce Mac; choisissez-en un autre. Rien n’est remplacé en silence.
    [invalid-settings] Ces réglages vidéo ne sont pas pris en charge (fréquence de 1 à 120, vitesse de 0,1 à 100, tailles paires).
    [empty-run] Cette sélection ne contient aucune photographie.
    [busy] Une autre vidéo est déjà en cours de création.
    [cancelled] La vidéo a été annulée; le fichier partiel a été supprimé.
    [ffmpeg-failed] FFmpeg s’est arrêté sur une erreur; le fichier partiel a été supprimé.
    [invalid-output] Le fichier terminé n’a pas passé la vérification (format, taille ou nombre d’images) et a été écarté.
    [output-exists] Un fichier portant ce nom existe déjà; rien n’a été écrasé.
    [image-unreadable] Une photographie de la sélection n’a pas pu être lue.
    [interrupted] L’application s’est fermée pendant la création de cette vidéo.
   *[other] La vidéo n’a pas pu être créée.
}
