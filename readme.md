# Poacher

**A log inspector TUI written in Rust.**

The goal of this project is to make a Text User Interface (TUI) tool that can easily parse nginx & 
apache log files and provide real-time stats. The secondary goal is for me (the author) to learn a low-level
programming language. That will most likely mean that this project will either never be finished or will be (quite) 
poor in quality (of code). 

### Features

Planned features are:

- Interactive TUI for easy navigation & viewing
- Ability to view the activity of a single site / log
- Ability to view the activity of a single IP across all logs
- Automatic detection of bad-actors in the logs (i.e. attacks, high request rate, etc)
- Ability to lookup an IP on AbuseIPDB

See process.txt for a psuedo-code / text version of the intended logic & parts.