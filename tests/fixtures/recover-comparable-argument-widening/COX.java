public class COX {
    static <T extends Comparable<T>> T pick(T a, T b){ return a.compareTo(b) >= 0 ? a : b; }
    static java.lang.Comparable big(java.math.BigInteger a, java.math.BigInteger b){ return pick(a, b); }  // java.math 不在 java.lang 表
}
