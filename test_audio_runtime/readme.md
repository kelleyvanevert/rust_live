# Livecoding language ideation

## Apprach

There's quite a big difference between:

- Arranging a structure _once_ (or gradually), and then sending it over to a _backend_ generator
  - Like for example Supercollider
- Somehow running the code _at the same time_
  - Maybe kinda like GUI immediate mode, then?

Although the second sounds kind of interesting, and may have more potential in terms of flexibility, the former is probably the easiest to prototype / make work.

(I guess that maybe a Processing-style `setup` vs `draw` could be a simple pragmatic compromise in the direction of the second approach?)

## Syntax

- Function-style, seems like the logical first choice, e.g.

  ```
  vocode(convolve(sin(440, type = square), sample), _)
  ```

- Method/builder-style?

  440.sin(type = square).convolve(sample).vocode(..)

- Graph-node style? Like fundsp, basically..

  440 -> sin[type = square]

- UI-style "blocks"?

  ```
  mix {
    freq = 440;
    sin { freq = 440, type[:t] = square };
  }

  convolve {
    in =
  }
  ```

## Modulation

- Optional single main argument w/o default, multiple named additional arguments, all w/ defaults

  ```
  sin[440]
  osc[freq = 440, type = square, squareness = 0.2]
  ```

- Map modulation names (`:bla`) to arguments in order to modulate at any depth

  ```
  let sound = sin[freq = :f, type = :ta] + sin[:f + sin(20), type = :tb];

  sound[:f = 460, :ta = [saw, square]@0.2, :tb = [saw, square]@0.2]
  ```

## Timelining / composition

- This is extremely important but also probably quite hard to get right:
  - These two seem quite contradictory:
    - As much flexibility as "copy and paste in Ableton"
    - But also the cool modulation & other things possible due to coding?

## Sound shaping

TODO
