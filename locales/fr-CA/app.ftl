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
home-root = Dossier source : { $path }

settings-section-privacy = Confidentialité
settings-privacy-description = Tout reste sur cet appareil. Rien n’est transmis.
erase-item-settings = Vos choix de langue, de thème et de densité

about-endorsed-by = par

## Erreurs (rendues via i18n/errors.ts → t(`error-${kind}`, args)).
error-import-root-unreadable = Le dossier « { $path } » n’a pas pu être lu.
error-image-unreadable = Une photographie située à « { $path } » n’a pas pu être hachée.
error-database = La base de données du projet n’a pas pu être ouverte ou écrite à l’emplacement « { $path } ».
error-store-transaction = L’enregistrement de l’import dans la base de données du projet a échoué.
