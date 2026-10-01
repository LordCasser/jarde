public class N1 {
    public enum Numbers {
        ONE((byte) 1, NumString.ONE), TWO((byte) 2, NumString.TWO);
        private final byte num;
        private final NumString str;
        public enum NumString {
            ONE("one"), TWO("two");
            private final String name;
            NumString(String name) { this.name = name; }
            public String getName() { return name; }
        }
        Numbers(byte n, NumString str) { this.num = n; this.str = str; }
        public int getNum() { return num; }
        public NumString getNumStr() { return str; }
        public String getName() { return str.getName(); }
    }
    public static void main(String[] args) {
        System.out.println(N1.Numbers.ONE.getNum());
        System.out.println(N1.Numbers.ONE.getNumStr());
        System.out.println(N1.Numbers.ONE.getName());
        System.out.println(N1.Numbers.TWO.getNum());
    }
}
