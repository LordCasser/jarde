public final class StringVarargsRunner {
    public static void main(String[] args) {
        String[] pair = StringVarargs.PAIR.valuesCopy();
        String[] single = StringVarargs.SINGLE.valuesCopy();
        String[] empty = StringVarargs.EMPTY.valuesCopy();
        if (pair.length != 2 || !".dex".equals(pair[0]) || !".class".equals(pair[1])
                || single.length != 1 || !".xml".equals(single[0]) || empty.length != 0
                || pair != StringVarargs.PAIR.valuesCopy()
                || single != StringVarargs.SINGLE.valuesCopy()
                || empty != StringVarargs.EMPTY.valuesCopy()
                || pair == single || pair == empty || single == empty) {
            throw new AssertionError();
        }
        System.out.println("OK StringVarargs");
    }
}
