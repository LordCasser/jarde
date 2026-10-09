public class NumberArgument {
    public static Number identity(Number value) {
        return value;
    }

    public static void main(String[] args) {
        System.out.println(identity(new java.math.BigDecimal("2.50")));
    }
}
