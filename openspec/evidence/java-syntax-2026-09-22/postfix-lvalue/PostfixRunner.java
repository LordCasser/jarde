public final class PostfixRunner {
    private static void expect(String name, String expected, String actual) {
        if (!expected.equals(actual)) {
            throw new AssertionError(name + ": expected " + expected + ", got " + actual);
        }
        System.out.println(name + "=" + actual);
    }

    private static void expectInt(String name, int expected, int actual) {
        expect(name, Integer.toString(expected), Integer.toString(actual));
    }

    private static String attempt(String name, Runnable action) {
        try {
            action.run();
            return "none";
        } catch (Throwable error) {
            return error.getClass().getSimpleName();
        }
    }

    public static void main(String[] args) {
        PostfixProbe.reset();
        int oldField = PostfixProbe.postReceiver();
        expectInt("receiver.old", 41, oldField);
        expectInt("receiver.new", 42, PostfixProbe.selected.value);
        expect("receiver.order", "R", PostfixProbe.trace);

        PostfixProbe.reset();
        int oldArray = PostfixProbe.postArray();
        expectInt("array.old", 70, oldArray);
        expectInt("array.new", 71, PostfixProbe.values[0]);
        expect("array.order", "AI", PostfixProbe.trace);

        PostfixProbe.reset();
        PostfixProbe.selected = null;
        expect("receiver.null.exception", "NullPointerException",
                attempt("receiver.null", new Runnable() {
                    public void run() { PostfixProbe.postReceiver(); }
                }));
        expect("receiver.null.order", "R", PostfixProbe.trace);

        PostfixProbe.reset();
        PostfixProbe.values = null;
        expect("array.null.exception", "NullPointerException",
                attempt("array.null", new Runnable() {
                    public void run() { PostfixProbe.postArray(); }
                }));
        expect("array.null.order", "AI", PostfixProbe.trace);

        PostfixProbe.reset();
        PostfixProbe.selectedIndex = 1;
        expect("array.bounds.exception", "ArrayIndexOutOfBoundsException",
                attempt("array.bounds", new Runnable() {
                    public void run() { PostfixProbe.postArray(); }
                }));
        expect("array.bounds.order", "AI", PostfixProbe.trace);

        PostfixProbe.reset();
        PostfixProbe.selected.value = Integer.MAX_VALUE;
        expectInt("overflow.old", Integer.MAX_VALUE, PostfixProbe.postReceiver());
        expectInt("overflow.new", Integer.MIN_VALUE, PostfixProbe.selected.value);

        PostfixProbe.reset();
        expectInt("local.increment", 9, PostfixProbe.localIncrement(8));

        PostfixProbe.INSTANCE.simpleField = 12;
        expectInt("simple-field.post.old", 12, PostfixProbe.INSTANCE.postSimpleField());
        expectInt("simple-field.post.new", 13, PostfixProbe.INSTANCE.simpleField);
        expectInt("simple-field.pre.result", 14, PostfixProbe.INSTANCE.preSimpleField());
        expectInt("simple-field.pre.new", 14, PostfixProbe.INSTANCE.simpleField);
    }
}
