import java.util.function.IntUnaryOperator;
public class DirectMethodRef { public static int abs(int n) { return ((IntUnaryOperator) Math::abs).applyAsInt(n); } }
