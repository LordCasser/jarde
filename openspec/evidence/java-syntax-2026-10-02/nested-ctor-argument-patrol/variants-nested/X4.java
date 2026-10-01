public class X4 {
    static class Leaf {
        final String s;
        Leaf(String s) { this.s = s; }
        public String toString() { return "Leaf(" + s + ")"; }
    }
    static class Mid {
        final Leaf l;
        Mid(Leaf l) { this.l = l; }
        public String toString() { return "Mid(" + l + ")"; }
    }
    static class Top {
        final Mid m;
        Top(Mid m) { this.m = m; }
        public String toString() { return "Top(" + m + ")"; }
    }
    static class Val {
        final String s;
        Val(String s) { this.s = s; }
        public String toString() { return "Val(" + s + ")"; }
    }
    static class Tag {
        final String tag;
        final Val v;
        Tag(String tag, Val v) { this.tag = tag; this.v = v; }
        public String toString() { return "Tag(" + tag + "," + v + ")"; }
    }
    public static String threeLayer() { return String.valueOf(new Top(new Mid(new Leaf("z")))); }
    public static String doubleUse() {
        Val kept;
        String r = String.valueOf(new Tag("t", kept = new Val("assigned")));
        return r + "/" + kept;
    }
    public static String crossBlock(boolean flag) { return String.valueOf(new Tag("t", flag ? new Val("a") : new Val("b"))); }
    public static void main(String[] args) {
        System.out.println(threeLayer());
        System.out.println(doubleUse());
        System.out.println(crossBlock(true));
    }
}
