package demo;

/* JADX INFO: loaded from: input-g.jar:demo/Op.class */
public enum Op {
    ADD { // from class: demo.Op.1
        @Override // demo.Op
        public int apply(int a, int b) {
            return a + b;
        }
    },
    MULTIPLY { // from class: demo.Op.2
        @Override // demo.Op
        public int apply(int a, int b) {
            return a * b;
        }
    };

    public abstract int apply(int i, int i2);

    public String tag() {
        return name() + ":" + ordinal();
    }
}
