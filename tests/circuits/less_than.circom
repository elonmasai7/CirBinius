pragma circom 2.1.0;

template LessThan(n) {
    signal input a;
    signal input b;
    signal output out;

    // out = 1 if a < b else 0
    component b2a = Num2Bits(n);
    var max = 1 << n;
    component comp = LessThan(n);
    comp.a <== a;
    comp.b <== b;
    out <== comp.out;
}

component main {public [a, b]} = LessThan(16);
