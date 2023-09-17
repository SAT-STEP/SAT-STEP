# SAT-STEP
[![CI](https://github.com//SAT-STEP/SAT-STEP/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com//SAT-STEP/SAT-STEP/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/SAT-STEP/SAT-STEP/graph/badge.svg?token=SJQY6B10OJ)](https://codecov.io/gh/SAT-STEP/SAT-STEP)

This project is for the University of Helsinki course Software engineering project.

## Description
An app for visualizing how [CaDiCal](https://github.com/arminbiere/cadical) SAT-solver works. Sudoku is used as the example problem. The goal is to show how the conflict driven algorithm learns new constraints while solving the problem.

## Links
[Product backlog and workhours](https://docs.google.com/spreadsheets/d/10uVJry0DMARkRh1FE6oqYXzprBYj8cL7fjpnKQULlAQ/edit?usp=sharing)

## Usage
- Make sure you have rust 1.72 installed.
- Clone this repository
- In the cloned repository: `cargo build -r`
- Now the app can be run: `cargo run`
