This is a rewrite of [fdired](https://github.com/Simar-malhotra09/fdired.git) in rust, because ofcourse!

Jokes aside, I was a little tired of how slow the progress was in C because of all the damn bug I tended to introduce with every commit,
and honestly, I couldn't be arsed to learn how to properly use a debugger at the moment.

Indeed, this project is also stained by the same human sloppness I excreated over at the C version; that, while still useable, will be archived soon.
Below is the readme of it for no reason other than to toot my own horn.

What else mmhh.. I'm kind of craving a sweet treat rn.

---

# fdired

Mix of fd and Emacs' Dired

<p align="center">
  <img src="assets/demo.gif" width="900">
</p>

## Original Scope

[fd](https://github.com/sharkdp/fd)

[Emacs' Dired](https://www.gnu.org/software/emacs/manual/html_node/emacs/Dired.html)

Never heard of Emacs or Dired? [Watch Tsoding's "The Annoying Usefulness of Emacs"](https://www.youtube.com/watch?v=DMbrNhx2zWQ&t=84s)

Why? Something like this probably exists already but I'm unemployed atm.

Why in C? I'm equally bad with all programming languages.

What is the nob.h file? [no-build](https://github.com/tsoding/nob.h)

## Current Scope

Supports

- Find
- Fd
- Grep
- Rg

Ideally we could support any filter that spits out newline seperated filepaths + metadata

# Why is the progress so slow?

### This is pure human slop 🦅🦅🦅

As mentioned before, I kinda suck at C and there no AI except for refactoring/checking vulnerabilities occasionally

# To do:

- [ ] Grep support: '|', ".", ".*"
- [ ] Be able to filter using vim's '/' syntax. Use n/N to navigate matches.
- [x] Be able to define how to open a file. Currently uses nvim for all files but obv you won't open a pdf that way.

Status?
(2026/09/12) Use ~/.config/fdired.txt to define how to open the supported filetypes.

(2026/07/24) Add basic functionality to open diff files with diff cmd. `nvim` for ascii, `open` for pdf etc. This now needs to be read from a dedicated config file, and tested.

(2026/07/05) Added additional func mapped to keys like <Tab>(toggle btw full filepath vs rel), <y>(copy filepath) etc. The parsing logic is terrible and breaks on "abc*". Will fix soon.

(2026/06/25) Added highlighting to UI. Line numbers in green, matched syntax in red, correct symbols for j/k keys. Fix general issues.

(2026/06/19) I've expanded the scope from just supporting fd to any utility that outputs atleast a newline seperated filepath (+ more like grep/rg).
Currently we can support find, fd, grep, rg. We inject some flags at runtime to ensure we get the output in the desired format.

(2026/06/01) Minimal UI improvements; add status bar at the bottom

(2026/05/31) Rendering is mostly done. Implemented viewport to handle lazy-loading for long fd output. Nav with j/k/gg/G works well.
Next need to implt <enter> to open file $EDITOR. And there needs to be someway to go back to view as well later.
Also will make the UI a bit better.

(2026/05/30) Basic render of fd output with ncurses supports j/k/gg/G nav; scrollable viewport not impl yet.

(2026/05/30) Just setup arg parsing. Atp fdired is just an alias for fd. Run as `./fdired [FLAG] [PATTERN] [PATH]`

## Building from source

```
git clone https://github.com/Simar-malhotra09/fdired
cd fdired
cc -o nob nob.c
./nob # just compiles the src/main.c
cd build # or wherever your build folder is, change in nob.c
./fdired [FLAG] [PATTERN] [PATH]

```
