public class UnknownArrayRelation {
    static class Base {}

    static final class Child extends Base {}

    interface UserMarker {}

    static final class MarkedChild implements UserMarker {}

    public static String accept(Base[] value) {
        return "Base[]:" + value.length;
    }

    public static String childToBase(Child[] value) {
        return accept(value);
    }

    public static String acceptMarker(UserMarker[] value) {
        return "UserMarker[]:" + value.length;
    }

    public static String childToMarker(MarkedChild[] value) {
        return acceptMarker(value);
    }
}
