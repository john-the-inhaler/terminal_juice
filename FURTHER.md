# FURTHER
## Changes that'll be coming to terminal-juice
### I'm writing them here because I'm not going to implement them now.

Currently, terminal-juice is barely implemented, this makes it easy to change.
The current setup isn't very good, primarily letting the programmer just hand
terminal-juice whatever file thingies they want isn't very simple. I'll be
changing it to be the simple "Grab stdio, unless i say otherwise" approach.

The transformation "interface" is neat, but I've implemented it the wrong way.
I want the transform inteface to be lazy and more flexible. It's nicer if you
could write code that describes initial setup before committing it and having
the terminal set up how you want it. Secondly, the transform objects are made
via the `Terminal` object. This is a good way of getting them, but the current
implementation has the transform bound to the `Terminal` that made it. This
seems reasonable, it means you can say `.commit()`, but I wan't transforms to
be seperate. You should still be able to derive a transform from a `Terminal`,
which will allow you to easily modify settings, but you can make a blank one
and then apply that.

So, after that rant, I'm going to list changes I want to make.
- decoupling Transform. (derive and new)
- make transforms composable not standalone (Inspired by Haskell)
- make user-controlled file choosing to not be the default.
- UTF-8 handling (better than u8)
- ANSI escape code handling (this might be complicated to integrate)
- bespoke impl for `Read`, `Write`, and `BufRead`.
  (This might allow us to use more "bufferfull" things without undue worry).

General plan (' ': not done, '.': doing, 'X': done):
1. [.] UTF8-handling (I'm in Wales, It's needed)
2. [ ] file-descriptor getting
3. [ ] custom `Write`, `Read`
4. [ ] ANSI escape code handling
