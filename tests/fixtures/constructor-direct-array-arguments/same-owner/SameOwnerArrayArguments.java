public final class SameOwnerArrayArguments {
    public static int ctorCalls;

    public int value;
    public SameOwnerArrayArguments[] children;

    public SameOwnerArrayArguments(int leaf) {
        ctorCalls++;
        this.value = leaf;
    }

    public SameOwnerArrayArguments(SameOwnerArrayArguments[] children) {
        ctorCalls++;
        this.children = children;
    }

    public static SameOwnerArrayArguments build() {
        return new SameOwnerArrayArguments(new SameOwnerArrayArguments[] {
            new SameOwnerArrayArguments(1),
            new SameOwnerArrayArguments(2)
        });
    }
}
