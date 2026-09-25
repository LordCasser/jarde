package demo;

/* JADX INFO: loaded from: subject.jar:demo/Op.class */
public enum Op {
    ADD { // from class: demo.Op.1
        @Override // demo.Op
        public int apply(int i, int i2) {
            return i + i2;
        }
    },
    MULTIPLY { // from class: demo.Op.2
        @Override // demo.Op
        public int apply(int i, int i2) {
            return i * i2;
        }
    };

    public abstract int apply(int i, int i2);

    public String tag() {
        return name() + ":" + ordinal();
    }
}
