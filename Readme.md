# PT
PT is a minimal pomodoro tracker with a command line interface. It supports task tracking and has a pomodoro timer.

## Installation

Use cargo to install PT.

```bash
cargo install --path .
mkdir ~/.pt
cp ./alarm.mp3 ~/.pt/alarm.mp3  # Or replace with whatever file you want to play when the timer ends.
```


## Usage

PT doesn't spawn any background processes, if you want to enable the timer functionality you need to run ```watch pt --notify``` in the background. Pt will then play a sound and display a desktop notification when a pomodoro has expired.

The main command line interface is as follows:
```bash

# Add a task
 ~> pt Make tea
Task 1 added.
001 [ ]: Make tea (Σ0 min)
# Start a pomodoro for task 1
 ~> pt -p 1
Pomodoro started for task 1.
001 [ ]: Make tea (24m 59s)
# Add another task
 ~> pt Water plants
Task 2 added.
001 [ ]: Make tea (24m 46s)
002 [ ]: Water plants (Σ0 min)
# Check off task 1
 ~> pt -c 1
Task 1 checked.
001 [x]: Make tea (24m 08s)
002 [ ]: Water plants (Σ0 min)
# Finish the pomodoro timer on task 1, tracking 1 minute of work
 ~> pt -f 1
Pomodoro finished for task 1.
001 [x]: Make tea (Σ1 min)
002 [ ]: Water plants (Σ0 min)
# Add another task
 ~> pt Meditate
Task 3 added.
001 [x]: Make tea (Σ0 min)
002 [ ]: Water plants (Σ0 min)
003 [ ]: Meditate (Σ0 min)
# Start pomodoros for tasks 2 and 3
 ~> pt -p 2 3
Pomodoro started for task 2.
Pomodoro started for task 3.
001 [x]: Make tea (Σ0 min)
002 [ ]: Water plants (24m 59s)
003 [ ]: Meditate (24m 59s)

```

A full list of commands can be found by running ```pt --help```.