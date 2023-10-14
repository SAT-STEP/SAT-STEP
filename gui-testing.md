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
    - Action 2
    - **Sub-Tests 3:**
        - Action 5
        - **Require:** Check 3
    - **Sub-Tests 4:**
        - Action 6
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
- Action 2
- Action 5
- **Check 3**
- Action 6
- **Check 4**

## Test cases
