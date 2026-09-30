public class P4LateSplit {
    public static String tag() { return "v"; }
    public static String g(boolean cond) {
        String early = tag() + "!";
        try {
            throw new IllegalStateException("state");
        } catch (IllegalStateException e) {
            StringBuilder sb = new StringBuilder();
            sb.append("a");
            if (cond) {
                sb.append("b");
            }
            System.out.println(sb.append("c").toString());
        }
        return early;
    }
    public static void main(String[] args) {
        System.out.println(g(false));
        System.out.println(g(true));
    }
}
