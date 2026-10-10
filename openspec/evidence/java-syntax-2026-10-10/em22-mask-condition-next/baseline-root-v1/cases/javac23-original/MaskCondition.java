public class MaskCondition {
    public int method3(int a, int b) {
        if (a + b < 10) {
            return a;
        }
        if ((a & b) != 0) {
            return a * b;
        }
        return b;
    }
}
