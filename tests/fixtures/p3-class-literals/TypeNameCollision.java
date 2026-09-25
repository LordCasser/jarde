// The class's name `java` makes the emitter's fully qualified spelling `java.lang.String` unsafe.
class java {
    static Class<?> target() {
        return String.class;
    }
}
