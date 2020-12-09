# cpy-walker

An experimental, remote CPython process memory walker.
This library enables you to connect to a CPython process and read its memory into Rust datatypes.

Currently (a subset of) CPython 2.7 is implemented.

## Example

The library only knows how to read objects and follow object pointers.
It cannot find objects by itself; you need to provide a memory address to start from.

```rust
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use cpy_walker::cpython27::*;
use cpy_walker::interpreter::*;
use cpy_walker::memory::{Memory, Process};
use cpy_walker::walker::walk;

/// This spawns a Python process, but you can connect to pre-exisitng processes as well.
fn spawn_child() -> Result<(i32, usize), Box<dyn std::error::Error>> {
    let child = Command::new(
        [env!("CARGO_MANIFEST_DIR"), "test-programs", "python27.py"]
            .iter()
            .collect::<PathBuf>(),
    )
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()?;

    let pid = child.id();
    let stdout = child.stdout.unwrap();

    let mut line = String::new();
    BufReader::new(stdout).read_line(&mut line)?;
    let pointer: usize = line.trim().parse().expect("memory address");

    Ok((pid as i32, pointer))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (pid, pointer) = spawn_child()?;
    let mem = cpy_walker::connect(pid)?;
    let ptr = Pointer::new(pointer);

    println!(
        "Data graph: {:#x?}",
        walk::<Cpython2_7, _>(&mem, ptr)
    );

    Ok(())
}
```

## Other crates

See also:
- [py-spy](https://github.com/benfred/py-spy) - a Python process profiler. It can find Python stacks and its locals. Useful to find memory adddress entry-points.

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/tomcur/cpy-walker/blob/master/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in cpy-walker by you, shall be licensed as MIT, without any additional
terms or conditions.
