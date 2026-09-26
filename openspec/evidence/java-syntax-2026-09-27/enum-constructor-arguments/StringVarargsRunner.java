public final class StringVarargsRunner {
    public static void main(String[] args) {
        if (StringVarargs.PAIR.valuesCopy().length != 2
                || !".class".equals(StringVarargs.PAIR.valuesCopy()[1])
                || StringVarargs.SINGLE.valuesCopy().length != 1
                || StringVarargs.EMPTY.valuesCopy().length != 0) throw new AssertionError();
        System.out.println("OK StringVarargs");
    }
}
