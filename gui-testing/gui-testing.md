# GUI testing

## Overview

Currently, the actual user-facing functionality (the GUI) is only covered by ad-hoc testing by developers, which is liable to miss problems.
It is also difficult to feel certain that all relevant cases have been covered.
The eventual goal is of course to find a good system for automatic GUI testing, but until then we should improve the reliability of manual testing.
The purpose of this document is to define test cases for manual testing in a way that should provide a good base for automatic testing later.
This way little effort is wasted, and testing is improved as soon as possible.
There should be tests for each user story, and new tests should be added at the same time as new features are merged into `main`.

## Structure

The tests will be organized into a nested list structure, where the higher level sections define "setup" actions for lower level ones.
Tests at the same level can be done without repeating the setup steps.
Tests are run in order.
Requirements are checked at the lowest level. For example the structure:


- Tests 1
    - Action 1
    - Action 2
    - **Sub-Tests 1:**
        - Action 3
        - **Require:** Check 1
    - **Sub-Tests 2:**
        - Action 4
        - **Require:** Check 2
- Tests 2
    - Action 1
    - **Sub-Tests 3:**
        - Action 5
        - **Require:** Check 3
    - Action 6
    - **Sub-Tests 4:**
        - Action 7
        - **Require:** Check 4

Would be run as:
- Action 1
- Action 2
- Action 3
- **Check 1**
- Action 4
- **Check 2**
- QUIT PROGAM
- Action 1
- Action 5
- **Check 3**
- Action 6
- Action 7
- **Check 4**

## Test cases

- Test basics
    - Start program with `cargo run`
    - Open file `data/sudoku1.txt` with the "Open file..." dialog
    - **Check initial state**
        - **Require:** There should be a sudoku grid with clues marked in a different style
        - **Require:** There should be row and col numbers shown beside the grid
        - **Require:** The sudoku grid should be as follows (barring style changes)
        ![Screenshot of initial state](initial.png "Initial sudoku state")
    - Solve sudoku with the "Solve sudoku" button
    - **Check solved state**
        - **Require:** The sudoku grid should be as follows (barring style changes)
        ![Screenshot of solved state](solved.png "Solved sudoku state")
        - **Require:** There should be a list of constraints
        - **Require:** There should be 482 constraints
        - **Require:** The start of the list should be as follows (barring style changes)
        ![Screenshot of start of list](list_start.png "First three constraints")
