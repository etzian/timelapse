/**
 * The project the user is working on: the source folder picked on Home. Held in memory
 * only (nothing is written to the device here), shared by Home and Cleaning.
 */
let root = $state('');

export const project = {
  get root(): string {
    return root;
  },
  set root(value: string) {
    root = value;
  }
};
