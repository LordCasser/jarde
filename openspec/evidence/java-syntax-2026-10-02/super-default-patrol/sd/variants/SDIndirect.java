public class SDIndirect {
    public interface SDA { default String name() { return "SA"; } }
    public interface SDM extends SDA { }
    public static class Use implements SDA {
        public static SDM probe;
        @Override public String name() { return SDA.super.name(); }
    }
    public static void main(String[] x) {
        System.out.println(new Use().name());
    }
}
