pragma circom 2.1.0;

template RangeCheck(n) {
    signal input in;
    signal output out;

    // Enforces 0 <= in < 2^n by constraining each bit
    signal bits[n];
    var acc = 0;
    for (var i = 0; i < n; i++) {
        bits[i] <-- (in >> i) & 1;
        bits[i] * (bits[i] - 1) === 0; // boolean constraint
        acc += bits[i] * (1 << i);
    }
    acc === in;
    out <== in;
}

component main = RangeCheck(8);
