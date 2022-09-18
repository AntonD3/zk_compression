# ZK Compression

![](mem.jpg)

## Motivation
Today there are a lot of rollups(both zk and optimistic) and the most valuable part of the transaction price - is fees for publishing calldata. And we are going to develop a toolset that will make rollups much cheaper.  

It will be the compression algorithm realized in zero-knowledge proofs. It can be easily integrated into any zk or optimistic rollup. It's a special compression algorithm that will work very effectively with storage transition data.

Of course, this algorithm can be used for any other tasks, as you can easily prove compression correctness with zero-knowledge proof.

## Tech specification
### Algorithm description
We want to make an algorithm that will be efficient with storage transition data. ZK rollups with contracts(or some other way for state interaction) usually use the next(or similar) format for storage transition record:
`(address, key, new_value)`.
#### Storage keys
But actually, most of the contracts are written on solidity and vyper. And they used the first storage slots for non-dynamic state variables. So actually most of the keys in storage transition are really small numbers(start with zeroes in binary format). We can encode the key like one byte for number of zero bytes and then non-zero part. When the number of zero bytes == 0 we can just write the key without such optimization.

Let's tell about dynamic arrays, strings, and bytes(Note: from this point, we will not explain what to do with strings, because strings are totally the same in storage with bytes). For dynamic arrays and bytes with size > 31, you can compute the slot of an element like `keccak256(small_slot_number) + some_small_value`, this two values can be compressed with the same algorithm as for static state variables(because they are usually so small). So you are saving the preimage of keccak256 and offset instead of the slot number.

The only data type not covered - is mappings, unfortunately for computing slot numbers you also should use keccak256, but preimage can be determined only at runtime. So it's a potential idea to improve, but for now not been implemented.

And some more info about nested dynamic types. We will not compress nested structures because they can be very expensive in circuits. In the future, we can support nesting for a few levels, but actually deep nesting - is not popular case.


#### Addresses

Address - the first bytes of `keccak256` output. So usually it's not a small number. But often a few storage slots can be changed in one transaction or can be just some popular contract whose address can be written many times in storage transitions data. So here good decision to use something like a dictionary, you can just map the address value to some number and then save the address in compressed data like a number which refers to the address.

#### Values

Case with values not so clear. But sometimes the value is so small (for example balance of some token, that uses uint256, but often it's just a few bytes). So we will use here the same algorithm as for keys.

#### Algorithm formalization

Let's introduce `type byte` term.
Every variable in encoded data will start with such byte and then you can uncompress data by this byte:
1. value == 0 - then it's just uncompressed 32-bytes value(key, value)
2. value == 1 - uncompressed 20-bytes value(address)
3. value == 2 - the next bytes contain two values `preimage` and `offset` which encoded like in 5-th type. And you can compute original values like `keccak256(preimage) + offset`.
4. value == 3 - the reference to some dictionary value. How dictionary value is stored will be explained below.
5. value >= 11 && value <= 42 - that's number which starts with `value - 10` zero bytes and in next `32 - (value - 10)` bytes non-zero suffix of data.

Some more notes:
- If for some value compression was non-effective, you will write it just like uncompressed (first 2 types).
- Dictionary values saving at the beginning. It's 20 bytes values (addresses). And first two bytes of all compressed data - number of dictionary words, then values. And after that compressed values.
- Note that this algorithm can be easily changed if the format or size of the field is other.
- There are a lot free values for type byte(4 - 10, 43 - 255). So the algorithm can be easily extended.

There are implementations of compression and uncompression.

#### Circuits implementation

We are going to prove that if we uncompress data it will be the same with some start data. Few points about implementation:
- Of course, the sizes of inputs will be limited. Two inputs will be public - hashes of compressed and uncompressed data
- We support ptr[N] variable which will refer to the compressed value. Once ptr[N] is uncompressed we know the ptr[N+1].
- Reading of uncompressing data will take O(n) constraints(n - size). So the number of constraints will be O(n^2). But in future it can be optimized with iterating over bytes.
- Prooving dictionary access can be done with merkle tree for O(log(n)) hashes, n - max number of values in the dictionary. The best hash for it - is `rescue`. But unfortunately, this part is not finished yet.

## Tests

For storage transitions for some simple contracts the compression can make data less for 70%. But actually for popular contracts(there are ERC20 example) we can remove 30% of data.

#### So this tool potentially can make rollup 30% cheaper!

## How to use it

For run tests for non-circuit part(it's includes effectivity tests) you should use:

`cargo test -- --nocapture`

For example with ZKP generation and validation:

`cargo run --bin circuits-test`
