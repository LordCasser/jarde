// A current declaration named `String` does not shadow the leading package component `java`.
class String {
    static Class<?> target() {
        return java.lang.String.class;
    }
}
