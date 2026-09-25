public final class PairRunner {
    public static void main(String[] args) {
        for (String text : new String[] { null, "x" }) {
            for (int mode : new int[] { 0, 1, 2 }) {
                ConstructorPairProbe value = new ConstructorPairProbe(text, mode);
                System.out.println(text + ":" + mode + ":" + value.first() + ":" + value.second());
            }
        }
    }
}
