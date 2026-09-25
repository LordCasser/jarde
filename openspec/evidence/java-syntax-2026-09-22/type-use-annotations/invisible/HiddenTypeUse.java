class HiddenTypeUse {
    java.lang.@HiddenMark("field") String field;

    java.lang.@HiddenMark("return") String value() {
        return "value";
    }

    String echo(java.lang.@HiddenMark("parameter") String value) {
        return value;
    }
}
