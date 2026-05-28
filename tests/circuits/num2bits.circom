pragma circom 2.1.0;

template Num2Bits(n) {
    signal input in;
    signal output out[n];

    var acc = 0;
    for (var i = 0; i < n; i++) {
        out[i] <-- (in >> i) & 1;
        out[i] * (out[i] - 1) === 0;
        acc += out[i] * (1 << i);
    }
    acc === in;
}

template Bits2Num(n) {
    signal input in[n];
    signal output out;

    out <== 0;
    for (var i = 0; i < n; i++) {
        in[i] * (in[i] - 1) === 0;
        out <== out + in[i] * (1 << i);
    }
}

component main = Num2Bits(8);
