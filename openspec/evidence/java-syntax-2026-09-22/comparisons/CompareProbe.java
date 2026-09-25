public class CompareProbe {
    public static int long_eq(long a, long b) { if (a == b) return 7; return 9; }
    public static int long_ne(long a, long b) { if (a != b) return 7; return 9; }
    public static int long_lt(long a, long b) { if (a < b) return 7; return 9; }
    public static int long_le(long a, long b) { if (a <= b) return 7; return 9; }
    public static int long_gt(long a, long b) { if (a > b) return 7; return 9; }
    public static int long_ge(long a, long b) { if (a >= b) return 7; return 9; }
    public static int long_not_lt(long a, long b) { if (!(a < b)) return 7; return 9; }
    public static int float_eq(float a, float b) { if (a == b) return 7; return 9; }
    public static int float_ne(float a, float b) { if (a != b) return 7; return 9; }
    public static int float_lt(float a, float b) { if (a < b) return 7; return 9; }
    public static int float_le(float a, float b) { if (a <= b) return 7; return 9; }
    public static int float_gt(float a, float b) { if (a > b) return 7; return 9; }
    public static int float_ge(float a, float b) { if (a >= b) return 7; return 9; }
    public static int float_not_lt(float a, float b) { if (!(a < b)) return 7; return 9; }
    public static int double_eq(double a, double b) { if (a == b) return 7; return 9; }
    public static int double_ne(double a, double b) { if (a != b) return 7; return 9; }
    public static int double_lt(double a, double b) { if (a < b) return 7; return 9; }
    public static int double_le(double a, double b) { if (a <= b) return 7; return 9; }
    public static int double_gt(double a, double b) { if (a > b) return 7; return 9; }
    public static int double_ge(double a, double b) { if (a >= b) return 7; return 9; }
    public static int double_not_lt(double a, double b) { if (!(a < b)) return 7; return 9; }
    public static int int_lt(int a, int b) { if (a < b) return 7; return 9; }
    public static boolean double_boolean(double a, double b) { return a < b; }
}
