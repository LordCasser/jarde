// Java resolves a class literal in type context; a local named `java` does not bind the package root.
class LocalNameQualifier {
    static Class<?> target() {
        int java = 1;
        Class<?> value = java.lang.String.class;
        return value;
    }
}
