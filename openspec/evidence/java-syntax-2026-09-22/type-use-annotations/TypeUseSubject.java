final class TypeUseSubject {
    @TypeMark("field") String field;

    @TypeMark("return") String value() {
        return "value";
    }

    String echo(@TypeMark("parameter") String value) {
        return value;
    }
}
