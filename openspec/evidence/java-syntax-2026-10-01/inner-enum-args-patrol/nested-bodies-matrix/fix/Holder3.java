package p;

public class Holder3 {
    public interface IMath {
        int compute(int x);
    }

    public enum Dual implements IMath {
        SQUARE {
            @Override
            public int compute(int x) { return x * x; }
            @Override
            public int extra() { return 4; }
        },
        NEGATE {
            @Override
            public int compute(int x) { return -x; }
            @Override
            public int extra() { return 5; }
        };

        public abstract int extra();
    }
}
