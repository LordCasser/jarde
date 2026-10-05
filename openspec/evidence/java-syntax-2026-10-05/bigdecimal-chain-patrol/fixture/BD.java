import java.math.*;
public class BD {
    static BigDecimal price(int units, BigDecimal rate){                       // 金额计算骨干形
        BigDecimal base = BigDecimal.valueOf(units).multiply(new BigDecimal("9.99"));
        return base.subtract(base.multiply(rate)).setScale(2, RoundingMode.HALF_UP);
    }
    static BigInteger fact(int n){                                             // BigInteger 累积（不可变乘）
        BigInteger r = BigInteger.ONE;
        for(int i = 2; i <= n; i++){ r = r.multiply(BigInteger.valueOf(i)); }
        return r;
    }
    static int compareChain(BigDecimal a, BigDecimal b){                       // compareTo 链（非 equals 陷阱）
        if(a.compareTo(b) < 0){ return -1; }
        else if(a.compareTo(b) > 0){ return 1; }
        return 0;
    }
    static BigDecimal divide(){                                                // 精度/舍入链
        return new BigDecimal("10").divide(new BigDecimal("3"), 4, RoundingMode.HALF_EVEN);
    }
    public static void main(String[] a){ System.out.println(""+price(3, new BigDecimal("0.10"))+"/"+fact(20)+"/"+compareChain(new BigDecimal("1.0"), new BigDecimal("1.00"))+"/"+divide()); }
}
