pragma circom 2.1.0;

// Merkle membership proof template (simplified)
// In production, this uses circomlib MerkleTreeChecker
template MerkleMembership(depth) {
    signal input leaf;
    signal input root;
    signal input siblings[depth];
    signal input indices[depth];

    // Each index must be boolean
    component isBool[depth];
    var computed = leaf;
    for (var i = 0; i < depth; i++) {
        indices[i] * (indices[i] - 1) === 0;
        // Hash pair: if indices[i]==0, pair = (computed, siblings[i])
        // else pair = (siblings[i], computed)
        // In production, call Poseidon/ SHA-256 hash here
        var left = indices[i] == 0 ? computed : siblings[i];
        var right = indices[i] == 0 ? siblings[i] : computed;
        computed = left + right; // simplified — real impl uses hash
    }
    computed === root;
}

component main {public [leaf, root]} = MerkleMembership(4);
