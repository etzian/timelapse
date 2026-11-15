## Chaînes propres à l’application. Les chaînes partagées (actions, erreurs,
## réglages, juridique, consentement) viennent de _foundation/i18n/common et
## se trouvent à côté de ce fichier.

-app-name = Timelapse Studio
app-name = { -app-name }
app-tagline = Transformez d’importants fichiers de photos en vidéos timelapse de haute qualité, localement.
app-endorsement = par Etzian
company-name = Etzian inc.

nav-home = Importer
nav-cleaning = Nettoyer
nav-organize = Organiser
nav-sampling = Sélectionner
nav-adjust = Ajuster
nav-export = Exporter
nav-sheets = Planches contact
nav-settings = Réglages
nav-group-workflow = Étapes
nav-group-tools = Outils
nav-group-app = Application
menu-settings = Réglages…
menu-file = Fichier
menu-open-folder = Ouvrir un dossier…
menu-edit = Édition
menu-window = Fenêtre
pipeline-state-not-started = non commencé
pipeline-state-in-progress = en cours
pipeline-state-done = terminé
pipeline-state-running = en exécution
pipeline-job-analysis = L’analyse
pipeline-job-adjust = L’ajustement
pipeline-job-export = L’exportation
pipeline-job-consolidation = Le déplacement des fichiers
pipeline-job-sheet = La planche contact
pipeline-job-ended = { $job } : terminé.
pipeline-job-started = { $job } a commencé.

home-import-action = Choisir un dossier à importer
home-importing = Importation…
home-welcome-title = Bienvenue
home-welcome-intro = Choisissez le dossier qui contient vos photographies. Tout reste sur cet appareil.
home-drop-hint = Ou déposez un dossier sur cette fenêtre.
home-drop-active = Relâchez pour ouvrir ce dossier.
home-drop-busy = Une importation est déjà en cours. Déposez le dossier de nouveau quand elle sera terminée.
home-rescan = Réanalyser
home-switch = Changer de projet
home-steps-title = Votre projet
home-steps-error = Les chiffres du projet n’ont pas pu être lus.
home-step-photos = Photographies
home-step-flagged = Signalées pour révision
home-step-not-analyzed = Pas encore analysées
home-step-selections = Sélections
home-step-exports = Exportations
home-step-open = Ouvrir
home-summary = { $new } nouvelle(s), { $unchanged } déjà dans le projet.
home-details-summary = Détails
home-stat-scanned = Fichiers explorés
home-stat-duplicates = Fichiers en doublon
home-hidden-removed = { $count ->
    [one] { $count } fichier macOS caché (une copie « ._ » des métadonnées d’une photo) importé plus tôt a été retiré du projet. Vos photographies ne sont pas touchées.
   *[other] { $count } fichiers macOS cachés (des copies « ._ » des métadonnées de photos) importés plus tôt ont été retirés du projet. Vos photographies ne sont pas touchées.
}
home-skipped-unreadable = { $count } fichier(s) n’ont pas pu être lus; ils ont été ajoutés sans vérification des doublons.
home-duplicate-groups-summary = Groupes de doublons trouvés lors de cet import
home-integrity-missing = { $count } photographie(s) du projet sont introuvables sur le disque
home-integrity-dups = { $count } groupe(s) de photographies identiques dans le projet

settings-section-privacy = Confidentialité
settings-privacy-description = Tout reste sur cet appareil. Rien n’est transmis.
erase-item-settings = Vos choix de langue, de thème et de densité, et votre liste de projets récents
erase-section = Choisir ce qu'il faut effacer
erase-section-description = Utilisez ensuite Effacer toutes les données ci-dessous. Vos photographies originales ne sont jamais touchées, sauf celles rejetées qui sont en quarantaine si vous la choisissez.
erase-choice-settings = Réglages de l'application
erase-choice-project = Données du projet
erase-choice-project-hint = Le dossier caché « .timelapse » de { $folder } ({ $size }) : vos décisions, sélections, analyses d’ajustement, aperçus, fichiers temporaires et historique d’annulation.
erase-choice-exports = Exportations
erase-choice-exports-hint = { $count ->
    [one] { $count } fichier ({ $size }) dans le dossier « Timelapse exports » : vidéos, leurs fiches et planches contact.
   *[other] { $count } fichiers ({ $size }) dans le dossier « Timelapse exports » : vidéos, leurs fiches et planches contact.
}
erase-choice-copies = Photographies copiées
erase-choice-copies-hint = { $count ->
    [one] { $count } copie ({ $size }) faite par Organiser dans { $folders }. Une copie que vous avez modifiée est conservée.
   *[other] { $count } copies ({ $size }) faites par Organiser dans { $folders }. Une copie que vous avez modifiée est conservée.
}
erase-choice-copies-moved = { $count ->
    [one] { $count } photographie déplacée par Organiser n’est jamais effacée : c’est le seul exemplaire.
   *[other] { $count } photographies déplacées par Organiser ne sont jamais effacées : ce sont les seuls exemplaires.
}
erase-choice-quarantine = Quarantaine
erase-choice-quarantine-hint = { $count ->
    [one] { $count } photographie originale rejetée ({ $size }) dans le dossier « Timelapse quarantine ». Elle sera supprimée définitivement.
   *[other] { $count } photographies originales rejetées ({ $size }) dans le dossier « Timelapse quarantine ». Elles seront supprimées définitivement.
}
erase-choice-empty = Rien à effacer.
erase-no-project = Ouvrez un dossier dans la page Importer pour aussi effacer ses données de projet, ses exportations, ses copies ou sa quarantaine.
erase-nothing-chosen = Rien : choisissez au moins un élément ci-dessus.
erase-item-project = Les données du projet de { $folder } ({ $size })
erase-item-exports = { $count ->
    [one] Les exportations : { $count } fichier ({ $size })
   *[other] Les exportations : { $count } fichiers ({ $size })
}
erase-item-copies = { $count ->
    [one] { $count } photographie copiée par Organiser ({ $size })
   *[other] { $count } photographies copiées par Organiser ({ $size })
}
erase-item-quarantine = { $count ->
    [one] { $count } photographie originale rejetée en quarantaine ({ $size })
   *[other] { $count } photographies originales rejetées en quarantaine ({ $size })
}
erase-done-copies-kept = { $count ->
    [one] { $count } copie a été conservée : elle a changé, est introuvable ou est maintenant un original.
   *[other] { $count } copies ont été conservées : elles ont changé, sont introuvables ou sont maintenant des originaux.
}
error-erase = { $reason ->
    [busy] Un traitement est en cours sur ce dossier. Attendez qu'il se termine ou arrêtez-le, puis effacez.
    [quarantine-inside] Des originaux rejetés sont encore dans les données du projet. Choisissez aussi la quarantaine, ou restaurez-les d'abord.
    [remove-failed] Un fichier n'a pas pu être supprimé ; il est peut-être ouvert dans une autre application. Fermez-le et réessayez.
   *[other] L'effacement n'a pas été mené à terme.
}
error-output = { $reason ->
    [unknown] Ce fichier ne figure plus dans le projet.
    [missing] Ce fichier n’est plus dans le dossier « Timelapse exports ». Il a peut-être été déplacé, renommé ou supprimé. Remettez-le à cet endroit ou exportez de nouveau la vidéo.
    [not-a-file] Ce n’est pas un fichier que l’application peut ouvrir.
    [outside-exports] Par sécurité, seuls les fichiers du dossier « Timelapse exports » de ce projet peuvent être ouverts.
    [open-failed] Le fichier n’a pas pu être ouvert. Vérifiez qu’une application de cet ordinateur peut lire des vidéos, ou affichez le fichier dans son dossier et ouvrez-le à partir de là.
    [unsupported-type] Seules les vidéos, leurs fiches et les planches contact créées par l’application peuvent être ouvertes.
    [unsupported-name] Le nom du fichier contient des caractères qui ne peuvent pas être transmis à une autre application. Renommez-le dans le Finder, puis réessayez.
   *[other] Le fichier n’a pas pu être ouvert.
}

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

## Phase 3 — nettoyage
clean-title = Nettoyer vos photographies
clean-intro = Timelapse Studio mesure la luminosité de chaque photographie et signale celles qui semblent trop sombres ou que Lightroom a marquées comme rejetées. Rien n’est retiré automatiquement : vous décidez, et une photo rejetée ne va dans la quarantaine du projet que lorsque vous le confirmez.
clean-no-project-title = Aucune collection ouverte
clean-no-project-description = Importez d’abord un dossier; ses photographies pourront ensuite être analysées et nettoyées ici.
clean-go-import = Aller à l’importation
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
clean-accept = Conserver
clean-done-accept = { $count ->
    [one] { $count } photographie conservée
   *[other] { $count } photographies conservées
}
clean-done-reject = { $count ->
    [one] { $count } photographie rejetée
   *[other] { $count } photographies rejetées
}
clean-done-clear = { $count ->
    [one] Décision effacée sur { $count } photographie
   *[other] Décisions effacées sur { $count } photographies
}
clean-undo = Annuler
clean-undone = Décisions rétablies.
clean-keys-title = Clavier
clean-keys-hint = La touche Tab donne accès aux photographies en une seule étape; les flèches se déplacent entre elles. Les boutons sur chaque photographie et dans la visionneuse font la même chose.
clean-cap-keep = A
clean-cap-reject = R
clean-cap-clear = U
clean-cap-open = Espace
clean-cap-select = X
clean-key-move = Se déplacer
clean-key-keep = Conserver
clean-key-reject = Rejeter
clean-key-clear = Effacer la décision
clean-key-open = Ouvrir la visionneuse
clean-key-select = Cocher ou décocher
clean-key-undo = Annuler la dernière décision
clean-grid-label = Photographies à trier
clean-photo-label = { $path }, { $status }
clean-inspector-position = { $index } de { $total }
clean-previous = Précédente
clean-next = Suivante
clean-inspector-keys = Flèches gauche et droite : précédente et suivante. Touche A : conserver; R : rejeter; U : effacer la décision.
clean-reject = Rejeter
clean-clear = Effacer la décision
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
sampling-title = Sélectionner les images
sampling-intro = Choisissez les photographies qui deviendront les images du timelapse. Les images suivent les dates de prise de vue, jamais l’ordre des fichiers; les photographies non retenues sont simplement laissées de côté, jamais supprimées.
sampling-no-project-description = Importez d’abord un dossier; vous pourrez ensuite choisir ses images ici.
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
sampling-all-dates = Toutes les dates
sampling-include-flagged = Inclure les photographies à examiner
sampling-input-note = Les photographies rejetées ou en quarantaine ne sont jamais utilisées. Les photographies sans date de prise de vue sont laissées de côté et comptées.
sampling-invalid = Vérifiez les valeurs : les nombres doivent être positifs (entiers pour les nombres de photos) et la date de fin ne peut pas précéder la date de début.
sampling-preview = Aperçu
sampling-previewing = Calcul en cours…
sampling-save = Enregistrer cette sélection
sampling-saving = Enregistrement…
sampling-saved = Enregistrée : { $label }.
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
sampling-verify-inputs-changed = Les photographies ont changé depuis cet enregistrement

## Phase 5 — exportation
export-title = Exporter la vidéo
export-intro = Transformez une sélection enregistrée en vidéo MP4. Elle est créée sur cet appareil et enregistrée dans le dossier « Timelapse exports » de votre dossier de photos; rien n’est écrasé.
export-no-project-description = Importez d’abord un dossier, puis enregistrez une sélection à la page Sélectionner.
export-no-ffmpeg-title = Cette application ne peut pas créer de vidéos
export-no-ffmpeg-description = L’outil vidéo fourni avec l’application est introuvable. Installez l’application à partir d’un paquet officiel pour créer des vidéos.
export-no-runs-title = Aucune sélection enregistrée
export-no-runs-description = Une vidéo est créée à partir d’une sélection enregistrée. Prévisualisez-en une et enregistrez-la à la page Sélectionner.
export-go-sampling = Aller à Sélectionner
export-settings-title = Vidéo
export-run = Sélection
export-codec = Format
export-codec-av1 = AV1 (recommandé)
export-codec-h264 = H.264 (le plus compatible)
export-codec-hevc = HEVC / H.265
export-codec-unavailable = { $codec } (non offert sur ce Mac)
export-codec-av1-help = L’AV1 produit les fichiers les plus légers à qualité égale et se lit dans les navigateurs récents, VLC et macOS 14 ou ultérieur.
export-codec-h264-help = Le H.264 se lit presque partout. Il est encodé par le matériel vidéo de votre Mac.
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
export-exporting = Création de la vidéo…
export-cancel = Annuler
export-cancelling = Annulation…
export-done-title = Vidéo créée
export-done = { $codec }, { $duration }, { $size }. Elle est enregistrée ici :
export-technical-details = Détails techniques
export-history-title = Vidéos produites dans ce projet
export-history-empty-title = Aucune vidéo pour l’instant
export-history-empty-description = Les vidéos que vous créez apparaissent ici avec leurs réglages et leur résultat.
export-status-running = En cours
export-status-completed = Terminée
export-status-failed = Échec
export-status-cancelled = Annulée
export-history-line = { $run }, { $codec }, { $duration }
about-ffmpeg-notice = L’encodage vidéo utilise FFmpeg (LGPL 2.1 ou ultérieure) et SVT-AV1 (licence BSD-3-Clause-Clear et Alliance for Open Media Patent License 1.0). Leurs textes de licence, réglages de compilation et emplacements des sources sont inclus dans l’application, dans Resources/resources/ffmpeg/LICENSES.

## Erreurs (rendues via i18n/errors.ts → t(`error-${kind}`, args)).
error-import-root-unreadable = Le dossier « { $path } » n’a pas pu être lu.
error-image-unreadable = La photographie située à « { $path } » n’a pas pu être lue.
error-database = Les données enregistrées du projet n’ont pas pu être ouvertes ou enregistrées dans « { $path } ».
error-store-transaction = Vos modifications n’ont pas pu être enregistrées dans le projet.
error-image-undecodable = La photographie située à « { $path } » n’est pas une image lisible.
error-unknown-image = Cette photographie ne fait plus partie du projet.
error-invalid-sampling = { $reason ->
    [too-many-targets] Cet intervalle est trop court pour la période : il créerait plus de cinq millions de pas.
   *[other] Les réglages d’échantillonnage doivent être des nombres positifs.
}
error-unknown-run = Cette sélection enregistrée n’existe plus.
error-export = { $reason ->
    [ffmpeg-missing] L’outil vidéo fourni avec l’application est introuvable.
    [encoder-unavailable] Ce format ne peut pas être encodé sur ce Mac; choisissez-en un autre. Rien n’est remplacé en silence.
    [invalid-settings] Ces réglages vidéo ne sont pas pris en charge (fréquence de 1 à 120, vitesse de 0,1 à 100, dimensions paires).
    [empty-run] Cette sélection ne contient aucune photographie.
    [busy] Une autre vidéo est déjà en cours de création.
    [cancelled] La vidéo a été annulée; le fichier partiel a été supprimé.
    [ffmpeg-failed] L’outil vidéo s’est arrêté sur une erreur; le fichier partiel a été supprimé.
    [invalid-output] Le fichier terminé n’a pas passé la vérification (format, taille ou nombre d’images) et a été écarté.
    [output-exists] Un fichier portant ce nom existe déjà; rien n’a été écrasé.
    [image-unreadable] Une photographie de la sélection n’a pas pu être lue.
    [write-failed] La vidéo n’a pas pu être écrite dans le dossier d’exportation du projet (disque plein ou non accessible en écriture).
    [interrupted] L’application s’est fermée pendant la création de cette vidéo.
    [unknown-export] Cette vidéo n’existe plus.
    [adjustment-stale] Une photographie de cette sélection a changé depuis l’analyse du scintillement ou de la stabilisation choisie. Analysez-la de nouveau à la page Ajuster.
    [adjustment-mismatch] Une analyse du scintillement ou de la stabilisation choisie appartient à une autre sélection, ou ne couvre pas toutes les photographies de celle-ci.
    [record-write-failed] Le registre n’a pas pu être enregistré dans le dossier d’exportation du projet (disque plein ou non accessible en écriture).
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

organize-title = Organiser vos fichiers
organize-no-project-description = Importez d’abord un dossier de photographies; vous pourrez ensuite les regrouper dans un dossier de travail.
organize-intro = Regroupez les photographies des dossiers de la collection dans un seul dossier de travail, en copies ou en déplaçant les originaux. Une simulation montre exactement ce qui se passera avant toute écriture, chaque fichier est vérifié, rien n’est jamais écrasé et chaque copie ou déplacement peut être annulé.
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
organize-confirm-safety = Chaque fichier est vérifié avant et après; rien n’est jamais écrasé, et l’opération peut être annulée depuis l’historique.
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
organize-originals-help = Relit chaque original à l’endroit indiqué par le projet et le compare à son état lors de l’importation.
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
sheets-intro = Une planche contact est un PDF de vignettes avec leur nom de fichier et leur heure de prise de vue, créé à partir de votre projet. Elle est enregistrée dans le dossier « Timelapse exports/contact-sheets » de votre dossier de photos.
sheets-source-title = Quoi afficher
sheets-source = Photographies
sheets-source-all = Toutes les photographies
sheets-source-period = Une période
sheets-source-flagged = Photographies signalées
sheets-source-quarantined = Photographies en quarantaine
sheets-source-sampling = Une sélection enregistrée
sheets-source-export = La séquence d’une vidéo
sheets-source-period-line = Du { $from } au { $to } (UTC)
sheets-source-sampling-line = Sélection nº { $id }
sheets-source-export-line = Vidéo nº { $id }
sheets-period-from = Du (UTC)
sheets-period-to = Jusqu’au, exclu (UTC)
sheets-period-help = Les photographies prises à partir de la première heure, jusqu’à la seconde exclue.
sheets-period-invalid = Choisissez un début et une fin; la fin doit suivre le début.
sheets-sampling-run = Sélection
sheets-no-runs = Aucune sélection enregistrée pour l’instant. Enregistrez-en une à la page Sélectionner.
sheets-export = Vidéo
sheets-export-option = Vidéo nº { $id } ({ $selection }, { $count } photographies)
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
export-record-help = Établi à partir de votre projet chaque fois qu’il est affiché; l’application ne le relit jamais.
export-record-label = Registre de la vidéo nº { $id }

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

## Phase 8 — réduction du scintillement et stabilisation

adjust-title = Ajuster
adjust-no-project-description = Importez d’abord un dossier de photographies et enregistrez une sélection; vous pourrez ensuite réduire le scintillement et les tremblements.
adjust-no-runs-description = La réduction du scintillement et la stabilisation s’appliquent à une sélection enregistrée. Enregistrez-en une à la page Sélectionner.
adjust-intro = Analysez une sélection enregistrée, vérifiez le résultat ici, puis choisissez-le au moment de l’exportation. Vos photographies ne sont jamais modifiées : les corrections sont appliquées pendant la création de la vidéo.
adjust-flicker-title = Réduction du scintillement
adjust-flicker-help = Suit les changements lents de la lumière (lever et coucher du soleil, nuages) et atténue les sauts d’exposition rapides d’une photographie à l’autre. Les corrections sont limitées à un diaphragme d’exposition et à un tiers de diaphragme de couleur.
adjust-strength = Intensité
adjust-window = Lissage
adjust-frames-option = { $count ->
    [one] { $count } image
   *[other] { $count } images
}
adjust-colour = Uniformiser aussi la couleur (balance des blancs)
adjust-analyse = Analyser
adjust-analysing = Analyse…
adjust-stopped = L’analyse a été arrêtée; rien n’a été enregistré.
adjust-other-running = Un autre projet est en cours d’analyse; vous pourrez commencer ici quand il sera terminé.
adjust-result = Résultat affiché
adjust-flicker-option = nº { $id } · { $date } · { $strength } · { $window } images
adjust-stab-option = nº { $id } · { $date } · recadrage { $crop }
adjust-stat-largest = Plus forte correction
adjust-stat-mean = Correction moyenne
adjust-stat-colour = Plus forte correction de couleur
adjust-stat-measured = Mesurées
adjust-measured-of = { $measured } sur { $frames }
adjust-stops = { $value } IL
adjust-none-title = Rien d’analysé pour l’instant
adjust-flicker-none = Analysez cette sélection pour voir sa luminosité, avant et après.
adjust-stab-none = Analysez cette sélection pour voir le mouvement de l’appareil, avant et après.
adjust-chart-brightness = Luminosité d’une image à l’autre (en diaphragmes, par rapport à la normale)
adjust-chart-red = Équilibre du rouge (en diaphragmes, par rapport à la normale)
adjust-chart-blue = Équilibre du bleu (en diaphragmes, par rapport à la normale)
adjust-chart-horizontal = Position horizontale (part de l’image)
adjust-chart-vertical = Position verticale (part de l’image)
adjust-series-before = Telle que prise
adjust-series-trend = Tendance conservée
adjust-series-after = Corrigée
adjust-series-path = Appareil tel que pris
adjust-series-stabilized = Stabilisée
adjust-axis-frame = Image
adjust-show-table = Afficher les valeurs dans un tableau
chart-value = { $label } : { $value }
chart-no-value = —
adjust-stab-title = Stabilisation
adjust-stab-help = Mesure le déplacement de l’image d’une photographie à l’autre et maintient l’appareil sur une trajectoire régulière. Déplacements seulement (pas de rotation); chaque image est recadrée de la même petite marge pour que les bords ne paraissent jamais.
adjust-smoothing = Lissage
adjust-max-crop = Recadrage maximal par côté
adjust-stat-crop = Recadrage par côté
adjust-stat-shift = Plus grand déplacement
adjust-stat-pairs = Mouvements mesurés
adjust-stab-few = Peu de mouvements ont pu être mesurés avec confiance (photographies sans détails ou très différentes) : la stabilisation aura peu d’effet sur cette sélection.
adjust-compare-title = Avant et après
adjust-compare-none = Analysez le scintillement ou la stabilisation pour comparer une photographie à sa version corrigée.
adjust-frame = { $count ->
    [one] Image (sur { $count })
   *[other] Image (sur { $count })
}
adjust-previous = Précédente
adjust-next = Suivante
adjust-apply-flicker = Réduction du scintillement
adjust-apply-stab = Stabilisation
adjust-label-before = Telle que prise
adjust-label-after = Corrigée
adjust-as-shot = Image { $frame } telle que prise
adjust-adjusted = Image { $frame } corrigée
export-flicker = Réduction du scintillement
export-stab = Stabilisation
export-adjust-none = Aucune
export-flicker-option = nº { $id } · intensité { $strength }
export-stab-option = nº { $id } · recadrage { $crop }

error-adjust = { $reason ->
    [busy] Une autre analyse est déjà en cours.
    [empty-run] Cette sélection ne contient aucune photographie.
    [unknown-adjustment] Cette analyse n’existe plus.
    [adjustment-mismatch] Cette analyse appartient à une autre sélection, ou ne couvre pas toutes les photographies de celle-ci. Analysez de nouveau cette sélection.
    [adjustment-stale] Une photographie de cette sélection a changé depuis son analyse. Analysez de nouveau la sélection.
    [unknown-frame] Cette image ne fait pas partie de la sélection.
   *[other] L’analyse n’a pas pu être effectuée.
}

project-recent = Projets récents
project-open = Ouvrir
project-remove = Retirer de la liste
project-open-named = Ouvrir { $name }
project-remove-named = Retirer de la liste : { $name }
project-opening = Ouverture de votre projet…
project-unreachable = Ce dossier est inaccessible.
project-unreachable-hint = Il a peut-être été déplacé ou renommé, ou son disque n’est pas branché. Branchez-le et réessayez, ou retirez-le de la liste. Vos photographies ne sont pas touchées.
project-last-opened = Dernière ouverture : { $date }

# Sélections nommées et étape suivante (P5)
selection-label = { $method } · { $count ->
    [one] { $photos } photographie
   *[other] { $photos } photographies
} · { $date }
selection-every-seconds = Toutes les { $n } s
selection-every-minutes = Toutes les { $n } min
selection-every-hours = Toutes les { $n } h
selection-every-days = { $count ->
    [one] Chaque jour
   *[other] Tous les { $n } jours
}
selection-nth = Une sur { $n }
selection-uniform = Répartition égale
selection-with-id = { $label } (nº { $id })
sampling-run-id = Sélection nº { $id }
sheets-selection-unknown = sélection nº { $id }
next-action = Suivant : { $step }
next-reason-checking = Vérification du projet…
next-reason-no-photos = Importez d’abord des photographies.
next-reason-analysing = Attendez la fin de l’analyse.
next-reason-no-selection = Enregistrez d’abord une sélection.
next-reason-adjusting = Attendez la fin de l’ajustement.

# Aperçu en défilement (P6)
flipbook-title = Aperçu en défilement
flipbook-play = Lecture (Espace)
flipbook-pause = Pause (Espace)
flipbook-prev = Image précédente (flèche gauche)
flipbook-next = Image suivante (flèche droite)
flipbook-first = Première image (Début)
flipbook-last = Dernière image (Fin)
flipbook-scrub = Position dans la séquence
flipbook-frame = Image { $n } sur { $total }
flipbook-rate = { $fps } images par seconde
flipbook-buffering = Chargement des images…
flipbook-failed = Cette photographie n’a pas pu être affichée. La lecture se poursuit.
flipbook-empty-title = Rien à lire
flipbook-empty-description = Choisissez d’abord des photographies; l’aperçu les lit dans l’ordre.
flipbook-partial = Lecture des { $shown } premières photographies sur { $total }.

wipe-label = Séparateur avant et après
wipe-value = Avant { $before }, après { $after }
wipe-hint = Glissez le séparateur ou utilisez les flèches du clavier. Maj ou Page précédente et Page suivante le déplacent par plus grands pas.
adjust-loop-title = Lire la boucle
adjust-loop-help = Lit jusqu’à { $count } photographies autour de l’image choisie, pour voir le scintillement et les secousses comme ils bougeraient.
adjust-loop-adjusted = Lire avec les corrections appliquées
export-preview-title = Aperçu
export-preview-help = La sélection à la cadence et à la vitesse ci-dessus, telle que prise. Les corrections ne s’appliquent que dans la vidéo exportée.

## Presets and summaries
clean-preset-label = Sensibilité
clean-preset-lenient = Indulgente
clean-preset-balanced = Équilibrée
clean-preset-strict = Stricte
clean-preset-custom = Réglages personnalisés
clean-preset-lenient-help = Ne signale que les photographies nettement sombres.
clean-preset-balanced-help = Un bon point de départ pour la plupart des collections.
clean-preset-strict-help = Signale tout ce qui semble même un peu sombre.
clean-advanced = Réglages avancés
export-preset-label = Usage
export-preset-share = Partage
export-preset-archive = Archive
export-preset-master = Copie maître
export-preset-share-help = Se lit presque partout. Pleine HD, qualité standard.
export-preset-archive-help = Le fichier le plus petit, en pleine HD et en qualité standard.
export-preset-master-help = La meilleure qualité, en 4K. Un fichier volumineux.
export-preset-custom = Réglages personnalisés
export-preset-unavailable = Le réglage « { $preset } » nécessite { $codec }, qui n’est pas disponible sur ce Mac.
export-advanced = Réglages avancés
summary-line = { $photos ->
    [one] { $photos } photographie devient
   *[other] { $photos } photographies deviennent
} { $frames ->
    [one] { $frames } image
   *[other] { $frames } images
}, pour une durée de { $duration } à { $fps } images par seconde.
summary-empty = Aucune photographie n’est sélectionnée : il n’y a encore rien à lire.

## UX — progression et résultat
progress-remaining = Temps restant : environ { $time }
output-play = Lire
output-reveal-mac = Afficher dans le Finder
output-reveal = Afficher dans le dossier
