interface IndirectParent {
    default int value() { return 3; }
}

interface IndirectChild extends IndirectParent {}

final class RedundantIndirectNegative implements IndirectChild {
    int valueFromAncestor() { return IndirectParent.super.value(); }
}
