# Quality Time 

[![codecov](https://codecov.io/github/Kaendor/quality-time/branch/main/graph/badge.svg?token=R2G5I2VUZT)](https://codecov.io/github/Kaendor/quality-time)

This project is a CLI used to gain insight on your code.

It is based on Churn (number of tome a file was edited) and complexity (Cyclomatic complexity: number of branch in a function)

## Goals
Provide simple actionable metric.

## Usages

After launching the CLI in TUI mode, you can escape using `q`. Thi will maybe change in the future.

```
Command line tool to generate actionable metrics for priorizing refactors on your rust project
Usage: quality-time.exe [OPTIONS] --project-path <PROJECT>

Options:
  -o, --output <OUTPUT>
          Output style of the CLI

          Possible values:
          - std-out: Print the results in the terminal as a human readable table
          - tui:     DIsplay the results with a graph in a terminal application

  -p, --project-path <PROJECT>
          The path of the repository to analyse

  -h, --help
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information
```

## Todos
- [ ] Add all the tests
- [ ] Use local error type instead of expects
- [ ] Add ignoring file possible
- [x] Add repo path configurable
- [ ] Support more langages
- [ ] Add time range configurable
- [ ] Add shortcut display in TUI
- [ ] Add marks on the axis for the selected file

[CHANGELOGS](./CHANGELOG.md)