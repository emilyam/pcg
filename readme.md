# pcg
This is an implementation of a PRNG from the PCG family.
Specifically, it implements PCG-XSH-RS-64/32 (MCG).
For more information on the PCG family of PRNGs,
see https://www.pcg-random.org/paper.html

Although the specific algorithm implemented is
reasonably secure, this implementation has not been
thoroughly tested and should be assumed not to be secure.
It is not suitable for secure applications.
Use at your own peril.

# Example use
```
let seed: u64 = 12345; // or any u64 seed, to taste
let mut pcg = Pcg::seed_from_u64(seed);

let x = pcg.next_u32();

let mut other_pcg = pcg.new_stream();
let y = other_pcg.next_u32();

assert_ne!(x, y);
```
