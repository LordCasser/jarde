public final class ExtraEntryRunner {
    public static void main(String[] args) {
        for (int bits = 0; bits < 8; bits++) {
            System.out.println(bits + ":" + ExtraEntryProbe.check((bits & 1) != 0, (bits & 2) != 0, (bits & 4) != 0));
        }
    }
}
