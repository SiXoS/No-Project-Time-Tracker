# No Project Time Tracker

**Disclaimer: This is in pre-alpha and it is very likely that there are bugs.**

This is a small CLI written in rust to report project-less work time. I have written
this tool quite specifically for my needs but hopefully someone else can get use
out of it as well. The data is stored in an SQLite DB.

You report time on a daily basis and the tool keeps track of your flex time. There
is also a "smart add" feature which you can place in your .bashrc or similar. It 
will ask you to fill in days that you have not reported for or do nothing if
your reports are up to date. The tool also supports csv exports.

## Root help section
You can also get help for any of the sub commands.
```
No Project Time Tracker 0.1
Simon Lindhén; Github: SiXoS
Track your time in a comfortable environment without silly buttons and pictures! Change DB location with environment
variable NPTT_DB_LOCATION.

USAGE:
    no-project-time-tracker [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add-flex     Add additional flex for occasions that don't coincide with normal condition. For example if you get
                 double flex
    add-time     Add a new line in time tracking.
    help         Prints this message or the help of the given subcommand(s)
    list-flex    List flex lines. Shows current month by default.
    list-time    List time tracking lines. Shows current month by default.
    report       Get a time report.
    smart-add    Will allow you to interactively add time for the previous workday(s) that has no time reported.
                 This can be placed in your .bashrc for example. You will then be requested to add the time for
                 unreported days as soon as you open the terminal. Will not do anything if the previous workday has
                 a report.
```

## Installation

Right now it is not published anywhere as it is not that polished, so you will
have to clone it and install it from source. Requires [Rust and Cargo](https://www.rust-lang.org/tools/install).
```
git clone git@github.com:SiXoS/No-Project-Time-Tracker.git
cd no-project-time-tracker
cargo install --path .
```

### Configuration
The default location for the SQlite DB is `~/.nptt-db`. You can change the folder
with the environment variable `NPTT_DB_LOCATION`.

## Features

### Existing features
- Multiple time entries per day.
- Break time so you don't have to register one entry before lunch and one after.
- Smart time reporting designed to be placed in ~/.bashrc so you don't forget to record time.
- Adding arbitrary flex not connected to a specific time. Useful for when you
get extra flex or to register initial flex before you start using this tool.
- Generating CSV reports that work well with for example google sheets.
- Reporting anytime on weekends will give you that time as flex.
- Days not reported time on will not affect the flex bank. This is due to a lack of PTO support.

### Critical lacking features
These features don't exist but they really should and I will probably add them soon.
- Editing. If you make a typo it will stick. You can edit the SQLite DB manually if
you really need to change something. 

### Lacking features
- Keeping track of various PTO.
- 12 hour format (AM/PM)

## Known bugs
Right now I'm not aware of any bugs, that means none exist, right? 
