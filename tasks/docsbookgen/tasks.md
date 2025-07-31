# Tiffany Docsbookgen Crate Tasks

## Goal
Build the docsbook for the `tinkerbell` project, responsible for:
- rust code docs

## Tasks

- [ ] Update `docsbookgen` to generate docs from the `tinkerbell` codebase:
  - [ ] Ensure it includes all public APIs
  - [ ] Generate markdown files for each module
  - [ ] Integrate with existing `mdbook` structure
  - [ ] CSS sytling for the generated docs to match the project's branding
    - [ ] use style.css from the `mdbook` theme
    - [ ] match the following tailwind config:
    - ```json]
      {
        "theme": {
          "extend": {
             colors:
            {
              'cyber-black': '#0a0a12',
              'cyber-dark': '#151525',
              'cyber-gray': '#1e1e35',
              'cyber-blue': '#00ffff',
              'cyber-pink': '#ff00ff',
              'cyber-purple': '#9900ff',
              'cyber-yellow': '#ffff00',
              'cyber-green': '#00ff99',
            },
            fontFamily:
            {
              'orbitron': ['Orbitron', 'sans-serif'],
              'rajdhani': ['Rajdhani', 'sans-serif'],
            },
          }
        }
      }
    ```
- [ ] Ensure the generated docs are linked correctly in the `mdbook` navigation
- [ ] Add a task to regenerate the docs when code changes are made
- [ ] Implement a CI step to validate the docs generation
- [ ] Add tests to ensure the generated docs match the expected structure and content
- [ ] Document the usage of the `docsbookgen` crate in the project README
- [ ] Ensure the generated docs are formatted according to the project's style guidelines
- [ ] Add a task to update the `docsbookgen` crate version in the project dependencies
- [ ] Ensure the `docsbookgen` crate is included in the project's CI pipeline
