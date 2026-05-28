pragma circom 2.0;

template RangeCheck(n) {
    signal input in;
    signal input max_bits;

    // Decompose input into bits and range-check each limb
    signal bits[n];
    var sum = 0;
    for (var i = 0; i < n; i++) {
        bits[i] <-- (in >> i) & 1;
        bits[i] * (bits[i] - 1) === 0; // boolean constraint
        sum += bits[i] * (1 << i);
    }
    sum === in;
}

component main = RangeCheck(8);
