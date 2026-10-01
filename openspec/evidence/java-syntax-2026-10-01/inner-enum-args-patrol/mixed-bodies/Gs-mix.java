package p;

public enum Gs {
    ONE(Simple.A) {
        @Override
        public int id() {
            return 11;
        }
    },
    TWO(Simple.B) {
        @Override
        public int id() {
            return 22;
        }
    };

    private final Simple s;

    Gs(Simple s) {
        this.s = s;
    }

    public abstract int id();

    public Simple owner() {
        return s;
    }

    public static void main(String[] args) {
        System.out.println(ONE.id() + " " + ONE.owner());
        System.out.println(TWO.id() + " " + TWO.owner());
    }
}
