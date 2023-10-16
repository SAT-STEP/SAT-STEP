# Definition of done
- The code has been formatted witn rustfmt
- The code 'passes' a check by the rust-clippy linter
- The code is sufficiently automatically tested
  - Every acceptance criterion for a user story is checked by a test
  - Code coverage  at least 60%
- Errors are handled appropriately in the whole codebase

# Git branching
- `main` and `development`branches contain only code meeting the definition of done
- All code enters `main` through a pull request from `development`
- All active development happens in separate feature branches
- Feature branches are merged into `development` once done
  - Aim to have feature branches last a maximum of 8 work hours before being merged into main
- `development` is merged into `main` only after there are new completed features for users
