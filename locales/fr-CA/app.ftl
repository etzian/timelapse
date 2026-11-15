## Chaînes propres à l’application. Les chaînes partagées (actions, erreurs,
## réglages, juridique, consentement) viennent de _foundation/i18n/common et
## se trouvent à côté de ce fichier.

-app-name = Timelapse Studio
app-name = { -app-name }
app-tagline = Transformez d’importants fichiers de photos en vidéos timelapse de haute qualité, localement.
company-name = Etzian inc.

nav-home = Accueil
nav-cleaning = Nettoyage
nav-organize = Organisation
nav-sampling = Échantillonnage
nav-export = Exportation
nav-sheets = Planches contact
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
home-hidden-removed = { $count ->
    [one] { $count } fichier macOS caché (une copie « ._ » des métadonnées d’une photo) importé plus tôt a été retiré du projet. Vos photographies ne sont pas touchées.
   *[other] { $count } fichiers macOS cachés (des copies « ._ » des métadonnées de photos) importés plus tôt ont été retirés du projet. Vos photographies ne sont pas touchées.
}
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
timeline-granularity-week = Par semaine
timeline-granularity-month = Par mois
timeline-week-of = Semaine du { $date }
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
clean-stop = Arrêter
clean-stopping = Arrêt…
clean-stopped = { $count ->
    [one] Analyse arrêtée. Ce qui a été analysé est conservé; il reste { $count } photographie. Lancez l’analyse de nouveau pour continuer.
   *[other] Analyse arrêtée. Ce qui a été analysé est conservé; il reste { $count } photographies. Lancez l’analyse de nouveau pour continuer.
}
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
clean-bulk-label = Décider pour plusieurs photographies
clean-select-photo = Sélectionner { $path }
clean-select-all = Tout sélectionner ({ $count })
clean-select-none = Effacer la sélection
clean-selected = { $count ->
    [one] { $count } photographie sélectionnée
   *[other] { $count } photographies sélectionnées
}
clean-selected-beyond = { $count ->
    [one] dont { $count } non affichée ci-dessous
   *[other] dont { $count } non affichées ci-dessous
}
clean-bulk-accept = Conserver la sélection
clean-bulk-reject = Rejeter la sélection
clean-bulk-clear = Effacer leurs décisions
clean-bulk-accept-title = { $count ->
    [one] Conserver { $count } photographie?
   *[other] Conserver { $count } photographies?
}
clean-bulk-accept-body = { $count ->
    [one] Elle reste disponible pour l’échantillonnage et la vidéo, quoi que l’analyse ait proposé.
   *[other] Elles restent disponibles pour l’échantillonnage et la vidéo, quoi que l’analyse ait proposé.
}
clean-bulk-accept-action = { $count ->
    [one] Conserver { $count } photographie
   *[other] Conserver { $count } photographies
}
clean-bulk-reject-title = { $count ->
    [one] Rejeter { $count } photographie?
   *[other] Rejeter { $count } photographies?
}
clean-bulk-reject-body = { $count ->
    [one] Elle est exclue de l’échantillonnage et de la vidéo. Le fichier n’est ni déplacé ni supprimé.
   *[other] Elles sont exclues de l’échantillonnage et de la vidéo. Les fichiers ne sont ni déplacés ni supprimés.
}
clean-bulk-reject-action = { $count ->
    [one] Rejeter { $count } photographie
   *[other] Rejeter { $count } photographies
}
clean-bulk-clear-title = { $count ->
    [one] Effacer la décision sur { $count } photographie?
   *[other] Effacer les décisions sur { $count } photographies?
}
clean-bulk-clear-body = { $count ->
    [one] Elle revient à ce que propose l’analyse.
   *[other] Elles reviennent à ce que propose l’analyse.
}
clean-bulk-clear-action = { $count ->
    [one] Effacer { $count } décision
   *[other] Effacer { $count } décisions
}
clean-bulk-undo = Vous pouvez modifier ces décisions en tout temps.
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
files-failure-missing = la copie n’est plus là
files-failure-modified = la copie a été modifiée depuis sa création; elle a donc été conservée
files-failure-moved-since = la photographie a été déplacée de nouveau depuis (par exemple en quarantaine); elle a été laissée où elle est
files-failure-remove-failed = la copie n’a pas pu être supprimée; elle est toujours là
files-failure-now-an-original = un original a été déplacé à cet endroit depuis; il a donc été laissé là
files-failure-unrecorded-copy = l’application s’est fermée pendant le placement de ce fichier; ce qui se trouve à la destination a été laissé tel quel
files-failure-replaced = le fichier à cet endroit n’est plus la copie faite par l’application; il a donc été laissé là
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
    [write-failed] La vidéo n’a pas pu être écrite dans le dossier d’exportation du projet (disque plein ou non accessible en écriture).
    [interrupted] L’application s’est fermée pendant la création de cette vidéo.
    [unknown-export] Cette vidéo n’existe plus.
    [record-write-failed] Le registre n’a pas pu être enregistré dans le dossier des exportations du projet (disque plein ou protégé en écriture).
   *[other] La vidéo n’a pas pu être créée.
}
error-consolidation = { $reason ->
    [destination-invalid] Choisissez un dossier existant ou nouveau par son chemin complet (pas un fichier).
    [destination-overlaps-collection] Le dossier de travail doit être hors de la collection importée et ne pas la contenir : l’importation reprendrait les copies.
    [destination-unwritable] Le dossier de travail n’a pas pu être créé ou n’est pas accessible en écriture.
    [insufficient-space] Il n’y a pas assez d’espace libre dans le dossier de travail pour ces photographies.
    [plan-changed] Le dossier ou les photographies ont changé depuis la simulation. Relancez la simulation, puis confirmez.
    [nothing-to-do] Il n’y a rien à placer : chaque photographie est déjà dans le dossier de travail ou écartée.
    [busy] Une autre organisation ou annulation est déjà en cours.
    [unknown-consolidation] Cette organisation n’existe plus.
    [already-undone] Cette organisation a déjà été annulée.
   *[other] Les photographies n’ont pas pu être organisées.
}

organize-title = Organisation
organize-no-project-description = Importez d’abord un dossier de photographies; vous pourrez ensuite les regrouper dans un dossier de travail.
organize-intro = Regroupez les photographies des dossiers de la collection dans un seul dossier de travail, en copies ou en déplaçant les originaux. Une simulation montre exactement ce qui se passera avant toute écriture, chaque fichier est vérifié, rien n’est jamais écrasé et chaque opération peut être annulée.
organize-settings-title = Réglages
organize-destination = Dossier de travail
organize-destination-none = Aucun dossier choisi
organize-destination-choose = Choisir un dossier…
organize-destination-help = Choisissez un dossier hors de la collection importée, par exemple sur un autre disque.
organize-operation = Opération
organize-operation-copy = Copier (les originaux restent)
organize-operation-move = Déplacer les originaux
organize-operation-copy-help = Chaque photographie est copiée; sur un disque de Mac, la copie n’occupe aucun espace supplémentaire tant qu’elle n’est pas modifiée. Modifier une copie ne change jamais l’original.
organize-operation-move-help = Chaque original va dans le dossier de travail une fois sa copie vérifiée; le projet le suit. Vous pouvez le remettre en place avec Annuler.
organize-source = Photographies
organize-source-all = Toutes, sauf les rejetées et celles en quarantaine
organize-naming = Noms
organize-naming-preserve = Conserver la structure des dossiers
organize-naming-original = Noms d’origine, dans un seul dossier
organize-naming-folder-prefix = Nom du dossier + nom d’origine
organize-naming-sequence = Numéros de séquence
organize-naming-preserve-help = Les sous-dossiers de la collection sont recréés dans le dossier de travail.
organize-naming-original-help = Toutes les photographies vont dans un seul dossier sous leur propre nom; un nom déjà pris reçoit _2, _3…
organize-naming-folder-prefix-help = Toutes les photographies vont dans un seul dossier, nommées d’après leur dossier d’origine (par exemple 157GOPRO_G0010001.JPG).
organize-naming-sequence-help = Toutes les photographies vont dans un seul dossier sous les noms 000001.jpg, 000002.jpg… en ordre chronologique ou dans l’ordre de la sélection.
organize-preview = Simulation
organize-previewing = Planification…
organize-preview-title = Simulation
organize-stat-planned = À placer
organize-stat-duplicates = Doublons écartés
organize-stat-present = Déjà présentes
organize-stat-renamed = Renommées
organize-stat-bytes = Taille
organize-stat-free = Espace libre
organize-free-unknown = Inconnu
organize-excluded = Exclues : { $rejected } rejetées, { $quarantined } en quarantaine, { $missing } introuvables à l’endroit prévu.
organize-unsafe-names = { $count ->
    [one] { $count } photographie a un nom qui ne peut pas être utilisé de façon sûre; elle est écartée.
   *[other] { $count } photographies ont un nom qui ne peut pas être utilisé de façon sûre; elles sont écartées.
}
organize-items-title = { $shown } premières photographies sur { $total }
organize-notable-title = Écartées ou renommées ({ $count })
organize-item-renamed = Renommée : le nom était déjà pris
organize-item-duplicate = Écartée : identique à une photographie précédente
organize-item-already-present = Écartée : un fichier identique est déjà là
organize-item-unsafe = Écartée : son nom ne peut pas être utilisé de façon sûre
organize-run-copy = { $count ->
    [one] Copier { $count } photo
   *[other] Copier { $count } photos
}
organize-run-move = { $count ->
    [one] Déplacer { $count } photo
   *[other] Déplacer { $count } photos
}
organize-preview-stale = Les réglages ont changé depuis la simulation. Relancez-la pour voir ce qui se passera.
organize-busy-run = Organisation en cours…
organize-busy-undo = Annulation en cours…
organize-confirm-title-copy = Copier les photographies?
organize-confirm-title-move = Déplacer les originaux?
organize-confirm-body-copy = { $count ->
    [one] { $count } photographie sera copiée vers { $path }.
   *[other] { $count } photographies seront copiées vers { $path }.
}
organize-confirm-body-move = { $count ->
    [one] { $count } original sera déplacé vers { $path }. Il ne sera plus dans le dossier de la collection.
   *[other] { $count } originaux seront déplacés vers { $path }. Ils ne seront plus dans les dossiers de la collection.
}
organize-confirm-safety = Chaque fichier est vérifié avant et après; rien n’est jamais écrasé et l’opération peut être annulée depuis l’historique.
organize-confirm-move-understood = Je comprends que les originaux quitteront les dossiers de la collection
organize-result-copy = { $failed ->
    [0] { $placed } photographies copiées.
   *[other] { $placed } photographies copiées; { $failed } n’ont pas pu l’être (voir l’historique).
}
organize-result-move = { $failed ->
    [0] { $placed } originaux déplacés.
   *[other] { $placed } originaux déplacés; { $failed } n’ont pas pu l’être (voir l’historique).
}
organize-undo-result = { $failed ->
    [0] Annulé : { $undone } fichiers repris.
   *[other] { $undone } fichiers repris; { $failed } ont été laissés tels quels (modifiés ou déplacés depuis).
}
organize-history-title = Historique
organize-history-empty-title = Rien n’a encore été organisé
organize-history-empty-description = Chaque copie ou déplacement apparaît ici, avec une vérification et une annulation.
organize-history-line-copy = { $count ->
    [one] { $count } photographie copiée, { $naming }
   *[other] { $count } photographies copiées, { $naming }
}
organize-history-line-move = { $count ->
    [one] { $count } original déplacé, { $naming }
   *[other] { $count } originaux déplacés, { $naming }
}
organize-history-failed = { $count } en échec
organize-history-undone = { $count } annulés
organize-status-running = En cours
organize-status-completed = Terminé
organize-status-partial = Partiellement terminé
organize-status-cancelled = Annulé
organize-status-interrupted = Interrompu
organize-undo-status-undone = Annulé
organize-undo-status-partially-undone = Partiellement annulé
organize-check = Vérifier
organize-checking = Vérification…
organize-check-ok = { $count ->
    [one] Le fichier est intact.
   *[other] Les { $count } fichiers sont intacts.
}
organize-check-problems = { $ok } intacts, { $modified } modifiés, { $missing } manquants, { $moved } déplacés depuis.
organize-undo = Annuler
organize-undo-title = Annuler cette organisation?
organize-undo-body-copy = { $count ->
    [one] La copie faite dans { $path } sera supprimée.
   *[other] Les { $count } copies faites dans { $path } seront supprimées.
}
organize-undo-body-move = { $count ->
    [one] L’original sera remis de { $path } à son emplacement d’origine.
   *[other] Les { $count } originaux seront remis de { $path } à leur emplacement d’origine.
}
organize-undo-safety = Seuls les fichiers encore exactement tels qu’ils ont été créés sont repris; tout ce qui a changé depuis reste en place et est listé. Rien d’autre dans le dossier n’est touché.
organize-originals-title = Vérification des originaux
organize-originals-help = Relit chaque original à l’endroit indiqué par le projet et le compare à son empreinte prise à l’importation.
organize-originals-check = Vérifier les originaux
organize-originals-ok = Intacts
organize-originals-changed = Modifiés
organize-originals-missing = Manquants
organize-originals-unhashed = Non vérifiables
organize-originals-all-ok = { $count ->
    [one] L’original est intact.
   *[other] Les { $count } originaux sont intacts.
}
organize-originals-problems = { $changed } modifiés et { $missing } manquants. L’application n’écrit jamais dans un original : ils ont été modifiés ou retirés hors de l’application.
organize-problem-modified = Modifié depuis sa création
organize-problem-missing = Introuvable
organize-problem-moved-since = Déplacé de nouveau depuis
organize-problem-changed = Modifié depuis l’importation

## Phase 7 — planches contact, périodes de la chronologie, registre d’exportation

sheets-title = Planches contact
sheets-no-project-description = Importez d’abord un dossier de photographies; vous pourrez ensuite les imprimer en planches contact.
sheets-intro = Une planche contact est un PDF de vignettes avec leur nom de fichier et leur heure de prise de vue, établi à partir des registres du projet. Elle est enregistrée dans le dossier .timelapse/contact-sheets du projet.
sheets-source-title = Quoi afficher
sheets-source = Photographies
sheets-source-all = Toutes les photographies
sheets-source-period = Une période
sheets-source-flagged = Photographies signalées
sheets-source-quarantined = Photographies en quarantaine
sheets-source-sampling = Une sélection enregistrée
sheets-source-export = La séquence d’une vidéo
sheets-source-period-line = Du { $from } au { $to } (UTC)
sheets-source-sampling-line = Sélection no { $id }
sheets-source-export-line = Vidéo no { $id }
sheets-period-from = Du (UTC)
sheets-period-to = Jusqu’au, exclu (UTC)
sheets-period-help = Les photographies prises à partir de la première heure, jusqu’à la seconde exclue.
sheets-period-invalid = Choisissez un début et une fin; la fin doit suivre le début.
sheets-sampling-run = Sélection
sheets-no-runs = Aucune sélection enregistrée pour l’instant. Enregistrez-en une à la page Échantillonnage.
sheets-export = Vidéo
sheets-export-option = Vidéo no { $id } (sélection no { $run }, { $count } photographies)
sheets-no-exports = Aucune vidéo créée pour l’instant.
sheets-layout-title = Mise en page
sheets-page-size = Papier
sheets-page-a4 = A4
sheets-page-letter = Lettre
sheets-orientation = Orientation
sheets-orientation-portrait = Portrait
sheets-orientation-landscape = Paysage
sheets-columns = Colonnes
sheets-rows = Rangées
sheets-show-filename = Nom du fichier
sheets-show-timestamp = Heure de prise de vue
sheets-times-utc = Les heures de prise de vue sont imprimées en UTC, comme l’application les affiche.
sheets-preview-title = Première page
sheets-preview-none-title = Rien à prévisualiser pour l’instant
sheets-preview-none-description = Terminez le choix des photographies pour voir la première page.
sheets-preview-label = { $count ->
    [one] Première page de la planche, { $count } photographie
   *[other] Première page de la planche, { $count } photographies
}
sheets-preview-help = Survolez une vignette pour voir sa légende. Le PDF utilise la même grille.
sheets-stat-photos = Photographies
sheets-stat-pages = Pages
sheets-stat-per-page = Par page
sheets-empty-title = Aucune photographie ici
sheets-empty-description = Ces photographies ne sont pas dans le projet, ou aucune ne correspond.
sheets-generate = Créer la planche contact
sheets-generating = Création…
sheets-done-title = Planche contact créée
sheets-done = { $pages ->
    [one] { $pages } page
   *[other] { $pages } pages
}, { $count ->
    [one] { $photos } photographie
   *[other] { $photos } photographies
}, { $size }.
sheets-unreadable = { $count ->
    [one] { $count } photographie n’a pas pu être lue; son emplacement est indiqué sur la planche.
   *[other] { $count } photographies n’ont pas pu être lues; leurs emplacements sont indiqués sur la planche.
}
sheets-other-running = La planche contact d’un autre projet est en cours de création; vous pourrez créer celle-ci quand elle sera terminée.
sheets-cancelled = La planche contact a été annulée; le fichier partiel a été supprimé.
sheets-history-title = Planches contact créées
sheets-history-empty-title = Aucune planche contact pour l’instant
sheets-history-empty-description = Les planches que vous créez sont listées ici, avec leur fichier.
sheets-history-line = { $pages ->
    [one] { $pages } page
   *[other] { $pages } pages
}, { $count ->
    [one] { $photos } photographie
   *[other] { $photos } photographies
}
sheets-status-running = En cours
sheets-status-completed = Créée
sheets-status-failed = Échec
sheets-status-cancelled = Annulée
sheets-status-interrupted = Interrompue
sheets-pdf-title = Planche contact
sheets-pdf-source = { $count ->
    [one] { $source } · { $photos } photographie
   *[other] { $source } · { $photos } photographies
}
sheets-pdf-page = Page { $page } de { $pages }
sheets-pdf-note = Heures de prise de vue en UTC · Timelapse Studio
sheets-pdf-unreadable = Illisible

timeline-zoom-all = Toute la collection
timeline-zoom-help = Choisissez une barre pour voir ses photographies et l’agrandir.
timeline-pick-help = Choisissez une barre pour voir ses photographies.
timeline-period-title = Photographies de { $period }
timeline-period-count = { $count ->
    [one] { $photos } photographie dans cette période
   *[other] { $photos } photographies dans cette période
}
timeline-period-sheet = Planche contact de cette période
timeline-identical-none = Aucune photographie ne partage la même heure de prise de vue qu’une autre.
timeline-identical-title = { $groups ->
    [one] { $groups } heure de prise de vue partagée par plusieurs photographies ({ $photos } photographies)
   *[other] { $groups } heures de prise de vue partagées par plusieurs photographies ({ $photos } photographies)
}
timeline-identical-help = Souvent une rafale, parfois un appareil dont l’horloge n’était pas réglée, ou des copies. Rien n’est modifié.
timeline-identical-count = { $count ->
    [one] { $photos } photographie
   *[other] { $photos } photographies
}
timeline-identical-more = { $count ->
    [one] Et { $count } autre heure.
   *[other] Et { $count } autres heures.
}

clean-sheet-flagged = Planche contact des signalées
clean-sheet-quarantined = Planche contact de la quarantaine
sampling-run-sheet = Planche contact
export-sheet = Planche contact
export-record-show = Afficher le registre
export-record-hide = Masquer le registre
export-record-save = Enregistrer le registre
export-record-saved = Registre enregistré :
export-record-help = Établi à partir des registres du projet chaque fois qu’il est affiché; l’application ne le relit jamais.
export-record-label = Registre de la vidéo no { $id }

error-timeout = L’application a mis trop de temps à répondre. Le travail est peut-être encore en cours : patientez un instant, puis réessayez.
error-contact-sheet = { $reason ->
    [busy] Une autre planche contact est déjà en cours de création.
    [empty] Il n’y a aucune photographie à afficher pour ce choix.
    [too-many] Une planche contact peut afficher au plus 100 000 photographies; choisissez-en moins.
    [invalid-period] Choisissez un début et une fin; la fin doit suivre le début.
    [unknown-export] Cette vidéo n’existe plus.
    [source-changed] Les photographies ont changé depuis l’aperçu (par exemple un statut a changé). Vérifiez l’aperçu, puis créez la planche de nouveau.
    [write-failed] La planche contact n’a pas pu être enregistrée dans le dossier du projet (disque plein ou protégé en écriture).
    [output-exists] Un fichier de ce nom existe déjà; rien n’a été écrasé.
    [interrupted] L’application s’est fermée pendant la création de cette planche contact.
   *[other] La planche contact n’a pas pu être créée.
}
