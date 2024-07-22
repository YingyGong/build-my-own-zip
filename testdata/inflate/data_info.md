# `inflate` Test Data

## fixed-huffman-empty

Bitstream: `1 10 0000000`  
Uncompressed: (none)

## fixed-huffman-literals

`1 10 00110 000 10110 000 10111 111 11001 0000 1110 00000 111 111111 00 00000`  

Bitstream: `1 10 00110000 10110000 10111111 110010000 111000000 111111111 0000000`  
Uncompressed: `00 80 8F 90 C0 FF`

## fixed-huffman-non-overlapping-run

Bitstream: `1 10 00110000 00110001 00110010 0000001 00010 0000000`  
Uncompressed: `00 01 02 00 01 02`

## fixed-huffman-overlapping-run0

Bitstream: `1 10 00110001 0000010 00000 0000000`  
Uncompressed: `01 01 01 01 01`

## fixed-huffman-overlapping-run1

Bitstream: `1 10 10111110 10111111 0000011 00001 0000000`  
Uncompressed: `8E 8F 8E 8F 8E 8F 8E`

## dynamic-huffman-empty

Dynamic Huffman block:

```text
numCodeLen=19
    codeLenCodeLen = 0:0, 1:1, 2:0, ..., 15:0, 16:0, 17:0, 18:1
numLitLen=257, numDist=2
    litLenCodeLen = 0:1, 1:0, ..., 255:0, 256:1
    distCodeLen = 0:1, 1:1
Data: End
```

Bitstream:

```text
1 01  // header
00000 10000 1111  // code counts
000 000 100 000 000 000 000 000 000 000 000 000 000 000 000 000 000 100 000  // code length code
0 11111111 10101011 0 0 0  // codes
1  // data
```

Uncompressed: (none)

## dynamic-huffman-empty-no-distance-code

Dynamic Huffman block:

```text
numCodeLen=18
    codeLenCodeLen = 0:2, 1:2, 2:0, ..., 15:0, 16:0, 17:0, 18:1
numLitLen=257, numDist=1
    litLenCodeLen = 0:0, ..., 254:0, 255:1, 256:1
    distCodeLen = 0:0
Data: End
```

Bitstream:

```text
1 01  // header
00000 00000 0111  // code counts
000 000 100 010 000 000 000 000 000 000 000 000 000 000 000 000 000 010  // code length code
01111111 00101011 11 11 10  // codes
1  // data
```

Uncompressed: (none)

## dynamic-huffman-one-distance-code

Dynamic Huffman block:

```text
numLitLen=258, numDist=1, numCodeLen=18
codeLenCodeLen = 0:2, 1:2, 2:2, ..., 15:0, 16:0, 17:0, 18:2
Literal/length/distance code lengths: 0 2 #18+1111111 #18+1101001 1 2 1
Data: 01 #257 #0 #256
```

Bitstream:

```text
1 01  // header
10000 00000 0111  // code counts
000 000 010 010 000 000 000 000 000 000 000 000 000 000 000 010 000 010"  // code length code
00 10 111111111 111001011 01 10 01  // codes
10 11 0 0  // data
```
 
Uncompressed: `01 01 01 01`

## fixed-lengths-stress, fixed-distances-stress

These two data files check that your length and distance symbol decoding is correct.
