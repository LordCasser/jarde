public class SDAbstract {
    public interface SDB { default String name() { return "SB"; } }
    public interface SDC { String name(); }
    public static class Use implements SDC, SDB {
        @Override public String name() { return SDB.super.name(); }
    }
    public static void main(String[] x) {
        System.out.println(new Use().name());
    }
}
