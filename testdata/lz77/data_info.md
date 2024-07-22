# LZ77 Test cases

The all a's files are about what you expect.
`complicated-lz77` is also relatively simple, but with more characters.

The sliding window and length test is a bit trickier:

- The `bbb` at the start and end are too far apart to be in the same sliding window.
- The long string of `a`s should be chunked into sizes `<= 258`.
