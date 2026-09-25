public final class PlainRunner {
    public static void main(String[] args) {
        for (int mode = 0; mode < 4; mode++) {
            System.out.println(mode + ":" + PlainMultiCatch.choose(mode));
        }
    }
}
