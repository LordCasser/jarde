public class NEG {
    static int sa, sb;
    static int[] arr = new int[2];
    int ia = 0;

    // The copy's second value is consumed by a local store, not by a field store.
    static int mixed() { int x; x = sa = 7; return x; }

    // The copy's second value is consumed inside the expression the first store's value is part of.
    static int expr() { sa = (sb = 5) + 1; return sb; }

    // The copy's second value is consumed by an array store.
    static int arrayConsumer() { arr[0] = sa = 3; return arr[0]; }

    // A receiver copy whose source is a call: the text would call it twice.
    static void callReceiver() { holder().ia |= 1; }
    static NEG holder() { return INSTANCE; }
    static final NEG INSTANCE = new NEG();

    public static void main(String[] a) {
        System.out.println("" + mixed() + "/" + expr() + "/" + arrayConsumer() + "/" + sa);
        callReceiver();
        System.out.println(INSTANCE.ia);
    }
}
