public class ScvConcatReReadControls {
    static boolean[] flags = new boolean[2];

    public static String reRead(int v) {
        boolean hasA = (v & 0x01) != 0 && (v & 0x02) != 0;
        flags[0] = hasA;
        return hasA + ":";
    }
}
