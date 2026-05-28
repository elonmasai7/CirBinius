pragma circom 2.1.0;

// Poseidon hash template placeholder — requires circomlib
// Full Poseidon implementation in circomlib uses a round-based
// permutation with S-box (x^5) and MDS matrix multiplication.
// This template demonstrates the constraint pattern that the
// hash lowering pass handles.
template Poseidon(nInputs) {
    signal input inputs[nInputs];
    signal output out;

    // In production: instantiate circomlib Poseidon(nInputs)
    // For fixture purposes: the lowering pass detects the
    // "poseidon" hint in signal names.
    var i = 0;
    var acc = 0;
    while (i < nInputs) {
        acc += inputs[i];
        i += 1;
    }
    out <== acc;
}

component main {public [inputs]} = Poseidon(2);
