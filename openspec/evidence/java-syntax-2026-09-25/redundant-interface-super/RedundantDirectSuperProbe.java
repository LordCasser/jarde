interface RedundantParent {
    default int value() { return 3; }
}

interface RedundantChild extends RedundantParent {
    @Override default int value() { return 4; }
}

public final class RedundantDirectSuperProbe implements RedundantParent, RedundantChild {
    @Override public int value() { return RedundantChild.super.value(); }
}
