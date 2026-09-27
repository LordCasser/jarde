package em27;

public final class Probe {
    public static String direct() {
        return new String(new char[]{'a', 'b', 'c'});
    }

    public static String stored() {
        char[] chars = new char[]{'a', 'b', 'c'};
        return new String(chars);
    }

    public static String wrongDescriptor() {
        return new String(new char[]{'a', 'b', 'c'}, 0, 3);
    }

    public static Holder wrongOwner() {
        return new Holder(new char[]{'a', 'b', 'c'});
    }

    public static String effectful() {
        return new String(new char[]{'a', (char) tick(), 'c'});
    }

    public static String extraReader() {
        return new String(read(new char[]{'a', 'b', 'c'}));
    }

    public static String extraWrite() {
        char[] chars = new char[]{'a', 'b', 'c'};
        chars[0] = 'z';
        return new String(chars);
    }

    public static String secondConsumer() {
        char[] chars = new char[]{'a', 'b', 'c'};
        char ignored = chars[0];
        return new String(chars);
    }

    private static int tick() { return 98; }
    private static char[] read(char[] chars) { int ignored = chars.length; return chars; }
    public static final class Holder { Holder(char[] chars) {} }
}
