public class N2 {
    public enum Operation implements N2.IOperation {
        PLUS {
            @Override
            public int apply(int x, int y) { return x + y; }
        },
        MINUS {
            @Override
            public int apply(int x, int y) { return x - y; }
        }
    }
    public interface IOperation {
        int apply(int x, int y);
    }
    public static void main(String[] args) {
        System.out.println(N2.Operation.PLUS.apply(3, 4));
        System.out.println(N2.Operation.MINUS.apply(9, 2));
    }
}
