package p;

public class Holder {
    public enum Op {
        PLUS(1) {
            @Override
            public int apply(int a, int b) {
                return a + b;
            }
        },
        MUL(2) {
            @Override
            public int apply(int a, int b) {
                return a * b;
            }
        },
        ID(0);

        private final int k;

        Op(int k) {
            this.k = k;
        }

        public int apply(int a, int b) { return k; }

        public static void main(String[] args) {
            System.out.println(Holder.Op.PLUS.apply(3, 4));
            System.out.println(Holder.Op.MUL.apply(3, 4));
            System.out.println(Holder.Op.ID.apply(3, 4));
        }
    }
}
