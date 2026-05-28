pragma circom 2.1.0;

template Mux2() {
    signal input sel;
    signal input a[2];
    signal output out;

    sel * (sel - 1) === 0;
    out <== (1 - sel) * a[0] + sel * a[1];
}

template Mux4() {
    signal input sel[2];
    signal input a[4];
    signal output out;

    component m0 = Mux2();
    component m1 = Mux2();
    m0.sel <== sel[0];
    m0.a[0] <== a[0];
    m0.a[1] <== a[1];
    m1.sel <== sel[0];
    m1.a[0] <== a[2];
    m1.a[1] <== a[3];

    component mux = Mux2();
    mux.sel <== sel[1];
    mux.a[0] <== m0.out;
    mux.a[1] <== m1.out;
    out <== mux.out;
}

component main = Mux4();
