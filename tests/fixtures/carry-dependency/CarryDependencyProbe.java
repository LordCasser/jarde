public final class CarryDependencyProbe {
    public static int effects;
    public static String trace = "";
    public CarryDependencyProbe() {}
    public static CarryDependencyProbe make() { trace += "M"; return new CarryDependencyProbe(); }
    public int value() { trace += "V"; return 5; }
    public static int arm() { trace += "A"; return 2; }
    public static boolean test(boolean flag) { trace += "T"; return flag; }
    public static void consume(int value) { effects = value; }
    public static void referenceConsumer(boolean flag) {
        consume(make().value() + (test(flag) ? arm() : 3));
    }
    public void thisConsumer(boolean flag) {
        consume(this.value() + (test(flag) ? arm() : 3));
    }
    public static void parameterConsumer(CarryDependencyProbe receiver, boolean flag) {
        consume(receiver.value() + (test(flag) ? arm() : 3));
    }
}
