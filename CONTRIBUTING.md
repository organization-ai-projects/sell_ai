# Guide de Contribution

Merci de contribuer à ce projet ! Voici quelques règles et bonnes pratiques pour faciliter le processus de collaboration.

## Branches

- **Branche par défaut** : `dev`
  - Toutes les pull requests doivent être dirigées vers la branche `dev`.
  - La branche `main` est réservée aux versions stables et prêtes pour la production.

## Workflow Git

1. **Synchronisez votre branche locale `dev` avec le dépôt distant** :

   ```bash
   git checkout dev
   git pull origin dev
   ```

2. **Créez une nouvelle branche pour vos modifications** :

   ```bash
   git checkout -b <nom-de-votre-branche>
   ```

3. **Effectuez vos modifications et commitez-les** :

   ```bash
   git add .
   git commit -m "feat: description de votre modification"
   ```

4. **Poussez votre branche vers le dépôt distant** :

   ```bash
   git push origin <nom-de-votre-branche>
   ```

5. **Créez une pull request** :
   - Allez sur GitHub et ouvrez une pull request vers la branche `dev`.

## Tests et Qualité du Code

- Avant de soumettre une pull request, assurez-vous que :

  - Tous les tests passent :

    ```bash
    cargo test
    ```

  - Le code est formaté correctement :

    ```bash
    cargo fmt
    ```

  - Il n'y a pas de warnings Clippy :

    ```bash
    cargo clippy
    ```

## Revue de Code

- Les pull requests doivent être approuvées par au moins un mainteneur avant d'être fusionnées.
- Répondez rapidement aux commentaires pour accélérer le processus de revue.

Merci pour votre contribution ! 🚀
