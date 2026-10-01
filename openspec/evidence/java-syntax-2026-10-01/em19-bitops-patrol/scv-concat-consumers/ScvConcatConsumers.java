public class ScvConcatConsumers {
    public static String plain(int v) {
        boolean hasA = (v & 0x01) != 0 && (v & 0x02) != 0;
        return hasA + ":";
    }

    public static String boxedHead(int v) {
        boolean hasA = (v & 0x01) != 0 && (v & 0x02) != 0;
        return "" + hasA + ":";
    }

    public static String doubleChain(int v) {
        boolean hasA = (v & 0x01) != 0 && (v & 0x02) != 0;
        boolean hasB = (v & 0x04) != 0 || (v & 0x08) != 0;
        return hasA + ":" + hasB;
    }

    public static String singleTail(int v) {
        boolean hasA = (v & 0x01) != 0 && (v & 0x02) != 0;
        boolean hasB = (v & 0x04) != 0;
        return hasA + ":" + hasB;
    }

    public static void main(String[] a) {
        System.out.println(plain(3));
        System.out.println(boxedHead(3));
        System.out.println(doubleChain(5));
        System.out.println(singleTail(5));
    }
}
