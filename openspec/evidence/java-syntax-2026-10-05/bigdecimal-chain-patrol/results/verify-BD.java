public class BD extends java.lang.Object {
    public BD() {
        super();
        return;
    }

    static java.math.BigDecimal price(int arg0, java.math.BigDecimal arg1) {
        java.math.BigDecimal local2 = java.math.BigDecimal.valueOf((long) arg0).multiply(new java.math.BigDecimal("9.99"));
        return local2.subtract((java.math.BigDecimal) local2.multiply(arg1)).setScale(2, java.math.RoundingMode.HALF_UP);
    }

    static java.math.BigInteger fact(int arg0) {
        java.math.BigInteger local1;
        int local2;
        local1 = java.math.BigInteger.ONE;
        for (local2 = 2; local2 <= arg0; local2 = local2 + 1) {
            local1 = local1.multiply((java.math.BigInteger) java.math.BigInteger.valueOf((long) local2));
        }
        return local1;
    }

    static int compareChain(java.math.BigDecimal arg0, java.math.BigDecimal arg1) {
        if (arg0.compareTo(arg1) < 0) {
            return -1;
        } else if (arg0.compareTo(arg1) > 0) {
            return 1;
    } else {
            return 0;
    }
    }

    static java.math.BigDecimal divide() {
        return new java.math.BigDecimal("10").divide(new java.math.BigDecimal("3"), 4, java.math.RoundingMode.HALF_EVEN);
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("").append((java.lang.Object) price(3, new java.math.BigDecimal("0.10"))).append("/").append((java.lang.Object) fact(20)).append("/").append(compareChain(new java.math.BigDecimal("1.0"), new java.math.BigDecimal("1.00"))).append("/").append((java.lang.Object) divide()).toString());
        return;
    }
}
