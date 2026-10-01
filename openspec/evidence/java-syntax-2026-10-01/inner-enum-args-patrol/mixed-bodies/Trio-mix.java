package p;

public enum Trio {
    ONE(1, "one", (byte) 4) {
        @Override
        public String describe(int extra) {
            return "ONE:" + extra;
        }
    },
    TWO(2, "two", (byte) 5) {
        @Override
        public String describe(int extra) {
            return "TWO:" + extra;
        }
    };

    private final int code;
    private final String label;
    private final byte small;

    Trio(int code, String label, byte small) {
        this.code = code;
        this.label = label;
        this.small = small;
    }

    public abstract String describe(int extra);

    public static void main(String[] args) {
        System.out.println(ONE.describe(10) + " " + ONE.ordinal());
        System.out.println(TWO.describe(20) + " " + TWO.ordinal());
        System.out.println(ONE.name() + " " + TWO.name());
    }
}
