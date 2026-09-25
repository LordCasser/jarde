public final class Runner {
    public static void main(String[] args) {
        System.out.println(new ConstructorConditionalProbe(null, 7).value());
        System.out.println(new ConstructorConditionalProbe("x", 7).value());
        System.out.println(new ConstructorConditionalProbe("", -1).value());
        System.out.println(new ConstructorConditionalProbe("x", Integer.MAX_VALUE).value());
    }
}
