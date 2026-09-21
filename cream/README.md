### Sept 21st

We currently only retrived one path per line, or seem to.

Concrete example:
Result of a standalone grep query:

```
❯ grep -rnIE 'text' ~/Desktop/code/tem-particle-seg/backend/src/app/models/
/Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/backends/impls/sam/torch_sam.py:9:from contextlib import contextmanager
/Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/backends/impls/sam/torch_sam.py:207:@contextmanager
/Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/impls/rf_recovery.py:129:    particle's own texture is background. A user-marked region carries no
```

Initalize content normally:

```
o_stdout: Packed::new(i_stdout),

```

Output:

```
STDOUT
Content: (contains newlines? : yes!)
/Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/backends/impls/sam/torch_sam.py:9:from contextlib import contextmanager
/Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/backends/impls/sam/torch_sam.py:207:@contextmanager
/Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/impls/rf_recovery.py:129:    particle's own texture is background. A user-marked region carries no

Matches:
  /Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/backends/impls/sam/torch_sam.py:9:-
  /Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/backends/impls/sam/torch_sam.py:207:-
  /Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/impls/rf_recovery.py:129:-


```

Initalize after removing all newlines in content:

```
o_stdout: Packed::new_with_strip_newlines(i_stdout),

```

Output:

```
STDOUT
Content: (contains newlines? : no!)
/Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/backends/impls/sam/torch_sam.py:9:from contextlib import contextmanager/Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/backends/impls/sam/torch_sam.py:207:@contextmanager/Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/impls/rf_recovery.py:129:    particle's own texture is background. A user-marked region carries no
Matches:
  /Users/0saker/Desktop/code/tem-particle-seg/backend/src/app/models/backends/impls/sam/torch_sam.py:9:-
```

As we can see, it only finds the first match, which (probably) isn't what we want. We'd like to parse for all filenames I think!

What's weird tho is the following case; the same issue doesn't show up!
as-is(but the ehcoed text doesn't consist of a escaped newline char, echo must add it automatically at the end, I think):

```
❯ echo "edit src/main.rs:abc:then Cargo.toml." | cargo run
   Compiling cream v0.1.0 (/Users/0saker/Desktop/code/probe/oreo/cream)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.32s
     Running `target/debug/cream`
STDIN
Content: (contains newlines? : yes!)
edit src/main.rs:abc:then Cargo.toml.

Matches:
  src/main.rs:-:-
  Cargo.toml:-:-
```

stripped:

```
❯ echo "edit src/main.rs:abc:then Cargo.toml." | cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/cream`
STDIN
Content: (contains newlines? : no!)
edit src/main.rs:abc:then Cargo.toml.
Matches:
  src/main.rs:-:-
  Cargo.toml:-:-
```

just for sanity's sake, also make sure echo doesn't emit trailing newline:

```
❯ echo -n "edit src/main.rs:abc:then Cargo.toml." | cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/cream`
STDIN
Content: (contains newlines? : no!)
edit src/main.rs:abc:then Cargo.toml.
Matches:                                                                                                                                                                                                        src/main.rs:-:-
  Cargo.toml:-:-
```

Maybe relevent:
Usage Notes
read_to_string attempts to read a source until EOF, but many sources are continuous streams that do not send EOF. In these cases, read_to_string will block indefinitely. Standard input is one such stream which may be finite if piped, but is typically continuous. For example, cat file | my-rust-program will correctly terminate with an EOF upon closure of cat. Reading user input or running programs that remain open indefinitely will never terminate the stream with EOF (e.g. yes | my-rust-program).
