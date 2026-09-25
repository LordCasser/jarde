final class RawTypedForeachFails {
    static void visit(Iterable values) {
        for (String value : values) {
            System.out.println(value);
        }
    }
}
