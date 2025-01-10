== Rust Rewrite (v0.2.2 vs v0.1.6)

=== Show Help

4.7 times faster

Benchmark 1: nps -h
  Time (mean ± σ):     991.4 µs ±  96.8 µs    [User: 263.2 µs, System: 676.8 µs]
  Range (min … max):   827.3 µs … 1386.3 µs    2361 runs

Benchmark 1: ./nps.sh -h
  Time (mean ± σ):       4.7 ms ±   0.2 ms    [User: 2.4 ms, System: 2.4 ms]
  Range (min … max):     4.3 ms …   5.5 ms    540 runs

=== Refresh Packages

==== Flakes

1.47 times faster (mainly due to processing JSON output from `nix search`)

Benchmark 1: nps -e=true -r
  Time (mean ± σ):      1.446 s ±  1.058 s    [User: 0.992 s, System: 0.413 s]
  Range (min … max):    1.101 s …  4.456 s    10 runs

Benchmark 1: ./nps --experimental=true -r
  Time (mean ± σ):      2.130 s ±  0.011 s    [User: 1.673 s, System: 0.956 s]
  Range (min … max):    2.115 s …  2.155 s    10 runs

==== Channels

1.07 times slower (noise, bottlenecked through internet I/O)

Benchmark 1: nps -e=false -r
  Time (mean ± σ):      9.752 s ±  0.635 s    [User: 7.511 s, System: 1.402 s]
  Range (min … max):    8.862 s … 10.330 s    10 runs

Benchmark 1: ./nps --experimental=false -r
  Time (mean ± σ):      9.154 s ±  0.087 s    [User: 7.620 s, System: 1.503 s]
  Range (min … max):    9.076 s …  9.383 s    10 runs

=== Short Match

==== Channels

3 times faster

Benchmark 1: ./nps --experimental=false neovim-gtk
  Time (mean ± σ):      17.7 ms ±   0.6 ms    [User: 13.2 ms, System: 13.0 ms]
  Range (min … max):    16.5 ms …  20.0 ms    134 runs

Benchmark 1: nps -e=false -i=false neovim-gtk
  Time (mean ± σ):       5.9 ms ±   0.3 ms    [User: 0.8 ms, System: 5.1 ms]
  Range (min … max):     4.9 ms …   7.3 ms    321 runs

==== Flakes

4.6 times faster

Benchmark 1: ./nps --experimental=true neovim-gtk
  Time (mean ± σ):      17.0 ms ±   1.3 ms    [User: 11.7 ms, System: 13.8 ms]
  Range (min … max):    15.6 ms …  28.3 ms    98 runs

Benchmark 1: nps -e=true -i=false neovim-gtk
  Time (mean ± σ):       3.7 ms ±   0.3 ms    [User: 0.6 ms, System: 3.2 ms]
  Range (min … max):     2.8 ms …   4.8 ms    443 runs

=== Medium Match

==== Channels

3.2 times faster

Benchmark 1: ./nps --experimental=false neovim
  Time (mean ± σ):      21.1 ms ±   0.6 ms    [User: 14.9 ms, System: 15.5 ms]
  Range (min … max):    19.7 ms …  23.7 ms    115 runs

Benchmark 1: nps -e=false -i=false neovim
  Time (mean ± σ):       6.5 ms ±   0.4 ms    [User: 1.1 ms, System: 5.4 ms]
  Range (min … max):     5.6 ms …   8.1 ms    292 runs

==== Flakes

5.4 times faster

Benchmark 1: ./nps --experimental=true neovim
  Time (mean ± σ):      20.4 ms ±   0.6 ms    [User: 14.2 ms, System: 15.5 ms]
  Range (min … max):    19.4 ms …  22.3 ms    122 runs

Benchmark 1: nps -e=true -i=false neovim
  Time (mean ± σ):       3.8 ms ±   0.3 ms    [User: 0.6 ms, System: 3.3 ms]
  Range (min … max):     2.9 ms …   4.8 ms    445 runs

=== Long Match

==== Channels

10.5 times faster

Benchmark 1: ./nps --experimental=false e
  Time (mean ± σ):     917.4 ms ±  10.7 ms    [User: 731.6 ms, System: 403.3 ms]
  Range (min … max):   903.8 ms … 938.4 ms    10 runs

 Benchmark 1: nps -e=false -i=false e
  Time (mean ± σ):      87.1 ms ±   0.9 ms    [User: 58.3 ms, System: 28.2 ms]
  Range (min … max):    85.4 ms …  88.9 ms    33 runs

==== Flakes

12.1 times faster

Benchmark 1: ./nps --experimental=true e
  Time (mean ± σ):      1.182 s ±  0.010 s    [User: 0.929 s, System: 0.499 s]
  Range (min … max):    1.172 s …  1.204 s    10 runs

Benchmark 1: nps -e=true -i=false e
  Time (mean ± σ):      97.7 ms ±   1.1 ms    [User: 69.7 ms, System: 27.4 ms]
  Range (min … max):    96.0 ms … 100.4 ms    29 runs
