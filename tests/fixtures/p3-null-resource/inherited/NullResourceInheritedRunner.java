public final class NullResourceInheritedRunner {
    public static void main(String[] args) {
        NullResourceInheritedOnly.useNullResource();
        if (NullResourceInheritedOnly.bodyCalls() != 1) {
            throw new AssertionError("body=" + NullResourceInheritedOnly.bodyCalls());
        }
        System.out.println("inherited-resource:bodyCalls=1,closes=0");
    }
}
