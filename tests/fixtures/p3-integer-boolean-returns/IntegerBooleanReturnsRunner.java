public final class IntegerBooleanReturnsRunner {
    public static void main(String[] args) {
        int[] values = {0, 1, 2, 3, -1, -2, Integer.MIN_VALUE, Integer.MAX_VALUE};
        for (int value : values) System.out.println("direct:" + value + ":" + IntegerBooleanReturns.direct(value));
        IntegerBooleanReturns self = new IntegerBooleanReturns();
        for (int value : values) System.out.println("once:" + value + ":" + IntegerBooleanReturns.once(self, value) + ":" + self.calls);
        for (int value : values) {
            self.value = value;
            System.out.println("post:" + value + ":" + self.post() + ":" + self.value);
            self.value = value;
            System.out.println("pre:" + value + ":" + self.pre() + ":" + self.value);
        }
        for (int value : values) {
            System.out.println("sync:" + value + ":" + IntegerBooleanReturns.syncOn(self, value));
            System.out.println("syncMethod:" + value + ":" + IntegerBooleanReturns.sync(self, value));
        }
        try { IntegerBooleanReturns.syncOn(null, 1); }
        catch (Throwable error) { System.out.println("null:" + error.getClass().getName()); }
    }
}
